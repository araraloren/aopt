use std::marker::PhantomData;

use crate::arg::Args;
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
pub struct GuessOptCfg<'a> {
    pub args: &'a Args,

    pub clopt: CLOpt,
}

impl<'a> GuessOptCfg<'a> {
    pub fn new(args: &'a Args, clopt: CLOpt) -> Self {
        Self { args, clopt }
    }
}

#[derive(Debug)]
pub struct OptGuess<'a, S>(PhantomData<&'a S>)
where
    S: Set;

impl<'a, S> OptGuess<'a, S>
where
    S: Set,
{
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

impl<'a, S> Default for OptGuess<'a, S>
where
    S: Set,
{
    fn default() -> Self {
        Self(PhantomData::default())
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
        let args = cfg.args;
        let index = args.get_index();
        let count = args.len();
        let clopt = &cfg.clopt;

        match style {
            UserStyle::EqualWithValue => {
                if clopt.value.is_some() {
                    matches.push(
                        OptMatch::default()
                            .with_index(index)
                            .with_total(count)
                            .with_argument(clopt.value.clone())
                            .with_style(OptStyle::Argument)
                            .with_disable(clopt.disable)
                            .with_name(valueof("name", &clopt.name)?)
                            .with_prefix(valueof("prefix", &clopt.prefix)?),
                    );
                }
            }
            UserStyle::Argument => {
                if clopt.value.is_none() {
                    matches.push(
                        OptMatch::default()
                            .with_index(index)
                            .with_total(count)
                            .with_consume_arg(true)
                            .with_argument(args.get_next().cloned())
                            .with_style(OptStyle::Argument)
                            .with_disable(clopt.disable)
                            .with_name(valueof("name", &clopt.name)?)
                            .with_prefix(valueof("prefix", &clopt.prefix)?),
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
                                    .with_index(index)
                                    .with_total(count)
                                    .with_argument(Some(name_value.1.into()))
                                    .with_style(OptStyle::Argument)
                                    .with_disable(clopt.disable)
                                    .with_name(name_value.0.into())
                                    .with_prefix(valueof("prefix", &clopt.prefix)?),
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
                                        .with_index(index)
                                        .with_total(count)
                                        .with_argument(Some(Str::default()))
                                        .with_style(OptStyle::Combined)
                                        .with_disable(clopt.disable)
                                        .with_name(format!("{}", char).into())
                                        .with_prefix(valueof("prefix", &clopt.prefix)?),
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
                            .with_index(index)
                            .with_total(count)
                            .with_argument(Some(Str::default()))
                            .with_style(OptStyle::Boolean)
                            .with_disable(clopt.disable)
                            .with_name(valueof("name", &clopt.name)?)
                            .with_prefix(valueof("prefix", &clopt.prefix)?),
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
    args: &'a Args,
    name: Option<Str>,
    index: Option<usize>,
}

impl<'a> GuessNOACfg<'a> {
    pub fn new(args: &'a Args, name: Option<Str>, index: Option<usize>) -> Self {
        Self { args, name, index }
    }

    pub fn get_name_or(&self, index: usize) -> Str {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            self.args[index].clone()
        }
    }

    pub fn get_index_or(&self) -> usize {
        self.index.unwrap_or_else(|| self.args.get_index())
    }
}

#[derive(Debug)]
pub struct NOAGuess<'a, S>(PhantomData<&'a S>)
where
    S: Set;

impl<'a, S> NOAGuess<'a, S>
where
    S: Set,
{
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

impl<'a, S> Default for NOAGuess<'a, S>
where
    S: Set,
{
    fn default() -> Self {
        Self(PhantomData::default())
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
        let args = cfg.args;
        // get pos from cfg or default pos
        let pos = cfg.get_index_or();
        let name = cfg.get_name_or(pos);
        let count = args.len();

        match style {
            UserStyle::Main => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_index(pos)
                        .with_total(count)
                        .with_style(OptStyle::Main),
                );
            }
            UserStyle::Pos => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_index(pos + 1) // when style is pos, noa index is [1..=noa_count]
                        .with_total(count)
                        .with_style(OptStyle::Pos),
                );
            }
            UserStyle::Cmd => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_index(pos)
                        .with_total(count)
                        .with_style(OptStyle::Cmd),
                );
            }
            _ => {
                unimplemented!("Unsupport generate Process for Opt Style")
            }
        }

        Ok(mat.map(|v| Self::Process::new(Some(v))))
    }
}
