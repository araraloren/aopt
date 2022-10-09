use std::marker::PhantomData;

use crate::arg::args::ArgsIter;
use crate::arg::CLOpt;
use crate::opt::OptStyle;
use crate::proc::NOAMatch;
use crate::proc::NOAProcess;
use crate::proc::OptMatch;
use crate::proc::OptProcess;
use crate::set::Set;
use crate::Error;
use crate::Str;

/// User set option style used for generate [`Process`](crate::proc::Process).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserStyle {
    Main,

    /// NOA argument base on position.
    Pos,

    /// The first NOA argument.
    Cmd,

    /// Option set style like `--opt=value`, the value is set after `=`.
    EqualWithValue,

    /// Option set style like `--opt value`, the value is set in next argument.
    Argument,

    /// Option set style like `--i42`, the value set in the option string.
    EmbeddedValue,

    /// Option set style like `-abc`, thus set both boolean options `a`, `b` and `c`.
    CombinedOption,

    /// Option set style like `--bool`, only support boolean option.
    Boolean,

    Custom(u64),
}

pub trait Guess {
    type Config;
    type Process;

    fn guess(
        &mut self,
        style: &UserStyle,
        cfg: Self::Config,
    ) -> Result<Option<Self::Process>, Error>;
}

pub fn valueof(name: &str, value: &Option<Str>) -> Result<Str, Error> {
    let string = value.as_ref().ok_or_else(|| {
        Error::raise_error(format!("No value of {name}, please check your option"))
    })?;
    Ok(string.clone())
}

#[derive(Debug)]
pub struct GuessOptCfg {
    pub idx: usize,

    pub len: usize,

    pub arg: Option<Str>,

    pub clopt: CLOpt,
}

impl GuessOptCfg {
    pub fn new<T, I>(iter: &ArgsIter<I>, clopt: CLOpt) -> Self
    where
        I: Iterator<Item = T> + Clone,
        T: Into<Str>,
    {
        Self {
            idx: iter.idx(),
            len: iter.len(),
            arg: iter.arg().cloned(),
            clopt,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn arg(&self) -> Option<&Str> {
        self.arg.as_ref()
    }
}

#[derive(Debug)]
pub struct OptGuess<S>(PhantomData<S>);

impl<S> Default for OptGuess<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> OptGuess<S> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }

    fn bool2str(value: bool) -> Str {
        if value {
            Str::from("true")
        } else {
            Str::from("false")
        }
    }
}

impl<S> Guess for OptGuess<S>
where
    S: Set,
{
    type Config = GuessOptCfg;

    type Process = OptProcess<S>;

    fn guess(
        &mut self,
        style: &UserStyle,
        cfg: Self::Config,
    ) -> Result<Option<Self::Process>, Error> {
        let mut matches = vec![];
        let index = cfg.idx();
        let count = cfg.len();
        let clopt = &cfg.clopt;

        match style {
            UserStyle::EqualWithValue => {
                if clopt.value.is_some() {
                    matches.push(
                        OptMatch::default()
                            .with_idx(index)
                            .with_len(count)
                            .with_arg(clopt.value.clone())
                            .with_sty(OptStyle::Argument)
                            .with_dsb(clopt.disable)
                            .with_name(valueof("name", &clopt.name)?)
                            .with_pre(valueof("prefix", &clopt.prefix)?),
                    );
                }
            }
            UserStyle::Argument => {
                if clopt.value.is_none() {
                    matches.push(
                        OptMatch::default()
                            .with_idx(index)
                            .with_len(count)
                            .with_consume(true)
                            .with_arg(cfg.arg().cloned())
                            .with_sty(OptStyle::Argument)
                            .with_dsb(clopt.disable)
                            .with_name(valueof("name", &clopt.name)?)
                            .with_pre(valueof("prefix", &clopt.prefix)?),
                    );
                }
            }
            UserStyle::EmbeddedValue => {
                if clopt.value.is_none() {
                    if let Some(name) = &clopt.name {
                        if name.len() >= 2 {
                            let name_value = name.split_at(1);

                            matches.push(
                                OptMatch::default()
                                    .with_idx(index)
                                    .with_len(count)
                                    .with_arg(Some(name_value.1.into()))
                                    .with_sty(OptStyle::Argument)
                                    .with_dsb(clopt.disable)
                                    .with_name(name_value.0.into())
                                    .with_pre(valueof("prefix", &clopt.prefix)?),
                            );
                        }
                    }
                }
            }
            UserStyle::CombinedOption => {
                if clopt.value.is_none() {
                    if let Some(name) = &clopt.name {
                        if name.len() > 1 {
                            for char in name.chars() {
                                matches.push(
                                    OptMatch::default()
                                        .with_idx(index)
                                        .with_len(count)
                                        .with_arg(Some(Self::bool2str(!clopt.disable)))
                                        .with_sty(OptStyle::Combined)
                                        .with_dsb(clopt.disable)
                                        .with_name(format!("{}", char).into())
                                        .with_pre(valueof("prefix", &clopt.prefix)?),
                                );
                            }
                        }
                    }
                }
            }
            UserStyle::Boolean => {
                if clopt.value.is_none() {
                    matches.push(
                        OptMatch::default()
                            .with_idx(index)
                            .with_len(count)
                            .with_arg(Some(Self::bool2str(!clopt.disable)))
                            .with_sty(OptStyle::Boolean)
                            .with_dsb(clopt.disable)
                            .with_name(valueof("name", &clopt.name)?)
                            .with_pre(valueof("prefix", &clopt.prefix)?),
                    );
                }
            }
            _ => {
                unimplemented!("Unsupport generate Process for NOA Style")
            }
        }

        Ok((!matches.is_empty()).then(|| Self::Process::new(matches)))
    }
}

pub struct GuessNOACfg<'a> {
    iter: &'a [Str],
    index: usize,
    name: Str,
}

impl<'a> GuessNOACfg<'a> {
    pub fn new(iter: &'a [Str], name: Str, index: usize) -> Self {
        Self { iter, name, index }
    }

    pub fn idx(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> Str {
        self.name.clone()
    }
}

#[derive(Debug)]
pub struct NOAGuess<'a, S>(PhantomData<&'a S>);

impl<'a, S> Default for NOAGuess<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> NOAGuess<'a, S> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }

    fn bool2str(value: bool) -> Str {
        if value {
            Str::from("true")
        } else {
            Str::from("false")
        }
    }
}

impl<'a, S> Guess for NOAGuess<'a, S>
where
    S: Set,
{
    type Config = GuessNOACfg<'a>;

    type Process = NOAProcess<S>;

    fn guess(
        &mut self,
        style: &UserStyle,
        cfg: Self::Config,
    ) -> Result<Option<Self::Process>, Error> {
        let mat;
        let iter = cfg.iter;
        let pos = cfg.idx();
        let name = cfg.name();
        let count = iter.len();

        match style {
            UserStyle::Main => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_idx(pos)
                        .with_len(count)
                        .with_sty(OptStyle::Main)
                        .with_arg(Some(Self::bool2str(true))),
                );
            }
            UserStyle::Pos => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_idx(pos)
                        .with_len(count)
                        .with_sty(OptStyle::Pos)
                        .with_arg(Some(Self::bool2str(true))),
                );
            }
            UserStyle::Cmd => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_idx(pos)
                        .with_len(count)
                        .with_sty(OptStyle::Cmd)
                        .with_arg(Some(Self::bool2str(true))),
                );
            }
            _ => {
                unimplemented!("Unsupport generate Process for Opt Style")
            }
        }

        Ok(mat.map(|v| Self::Process::new(Some(v))))
    }
}
