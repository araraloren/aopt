use std::ffi::OsStr;
use std::ffi::OsString;
use std::marker::PhantomData;

use crate::arg::Args;
use crate::arg::CLOpt;
use crate::opt::OptStyle;
use crate::proc::NOAMatch;
use crate::proc::NOAProcess;
use crate::proc::OptMatch;
use crate::proc::OptProcess;
use crate::set::Set;
use crate::Arc;
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
pub struct GuessOptCfg<'a> {
    pub idx: usize,

    pub len: usize,

    pub arg: Option<Arc<OsString>>,

    pub clopt: &'a CLOpt,
}

impl<'a> GuessOptCfg<'a> {
    pub fn new(idx: usize, len: usize, arg: Option<Arc<OsString>>, clopt: &'a CLOpt) -> Self {
        Self {
            idx,
            len,
            arg,
            clopt,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn arg(&self) -> Option<&Arc<OsString>> {
        self.arg.as_ref()
    }
}

#[derive(Debug)]
pub struct OptGuess<'a, S>(PhantomData<&'a S>);

impl<'a, S> Default for OptGuess<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> OptGuess<'a, S> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }

    fn bool2str(value: bool) -> Arc<OsStr> {
        if value {
            OsString::from("true").into()
        } else {
            OsString::from("false").into()
        }
    }
}

impl<'a, S> Guess for OptGuess<'a, S>
where
    S: Set,
{
    type Config = GuessOptCfg<'a>;

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
                            .with_arg(cfg.arg().cloned())
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
                                    .with_arg(Some(OsString::from(name_value.1).into()))
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
                                        .with_arg(None)
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
                            .with_arg(None)
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

pub struct GuessNOACfg {
    index: usize,
    total: usize,
    args: Arc<Args>,
}

impl GuessNOACfg {
    pub fn new(args: Arc<Args>, index: usize, total: usize) -> Self {
        Self { args, index, total }
    }

    pub fn idx(&self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
        self.total
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

    fn bool2str(value: bool) -> Arc<OsStr> {
        if value {
            OsString::from("true").into()
        } else {
            OsString::from("false").into()
        }
    }
}

impl<'a, S> Guess for NOAGuess<'a, S>
where
    S: Set,
{
    type Config = GuessNOACfg;

    type Process = NOAProcess<S>;

    fn guess(
        &mut self,
        style: &UserStyle,
        cfg: Self::Config,
    ) -> Result<Option<Self::Process>, Error> {
        let mat;
        let args = cfg.args.clone();
        let pos = cfg.idx();
        let count = cfg.len();

        match style {
            UserStyle::Main => {
                mat = Some(
                    NOAMatch::default()
                        .with_args(args)
                        .with_idx(pos)
                        .with_len(count)
                        .with_sty(OptStyle::Main),
                );
            }
            UserStyle::Pos => {
                mat = Some(
                    NOAMatch::default()
                        .with_args(args)
                        .with_idx(pos)
                        .with_len(count)
                        .with_sty(OptStyle::Pos),
                );
            }
            UserStyle::Cmd => {
                mat = Some(
                    NOAMatch::default()
                        .with_args(args)
                        .with_idx(pos)
                        .with_len(count)
                        .with_sty(OptStyle::Cmd),
                );
            }
            _ => {
                unimplemented!("Unsupport generate Process for Opt Style")
            }
        }

        Ok(mat.map(|v| Self::Process::new(Some(v))))
    }
}
