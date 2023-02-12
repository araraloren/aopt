use std::marker::PhantomData;
use std::ops::Deref;

use crate::args::Args;
use crate::args::CLOpt;
use crate::opt::Style;
use crate::opt::BOOL_FALSE;
use crate::opt::BOOL_TRUE;
use crate::proc::NOAMatch;
use crate::proc::NOAProcess;
use crate::proc::OptMatch;
use crate::proc::OptProcess;
use crate::set::OptValidator;
use crate::set::Set;
use crate::ARef;
use crate::Error;
use crate::RawVal;
use crate::Str;

/// User set option style used for generate [`Process`](crate::proc::Process).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
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

    /// Option set style like `--i42`, the value set in the option string, only support one letter.
    EmbeddedValue,

    /// Option set style like `--opt42`, the value set in the option string, but suppport more than one letter.
    EmbeddedValuePlus,

    /// Option set style like `-abc`, thus set both boolean options `a`, `b` and `c`.
    CombinedOption,

    /// Option set style like `--bool`, only support boolean option.
    Boolean,
}

pub trait UserStyleManager {
    fn style_manager(&self) -> &OptStyleManager;

    fn style_manager_mut(&mut self) -> &mut OptStyleManager;
}

/// Manage the support option set style[`UserStyle`].
#[derive(Debug, Clone)]
pub struct OptStyleManager {
    styles: Vec<UserStyle>,
}

impl Default for OptStyleManager {
    fn default() -> Self {
        Self {
            styles: vec![
                UserStyle::EqualWithValue,
                UserStyle::Argument,
                UserStyle::Boolean,
                UserStyle::EmbeddedValue,
            ],
        }
    }
}

impl OptStyleManager {
    pub fn with(mut self, styles: Vec<UserStyle>) -> Self {
        self.styles = styles;
        self
    }

    pub fn set(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.styles = styles;
        self
    }

    pub fn remove(&mut self, style: UserStyle) -> &mut Self {
        if let Some((index, _)) = self.styles.iter().enumerate().find(|v| v.1 == &style) {
            self.styles.remove(index);
        }
        self
    }

    pub fn insert(&mut self, index: usize, style: UserStyle) -> &mut Self {
        self.styles.insert(index, style);
        self
    }

    pub fn push(&mut self, style: UserStyle) -> &mut Self {
        if !self.styles.iter().any(|v| v == &style) {
            self.styles.push(style);
        }
        self
    }
}

impl Deref for OptStyleManager {
    type Target = Vec<UserStyle>;

    fn deref(&self) -> &Self::Target {
        &self.styles
    }
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

/// Guess configuration for option.
#[derive(Debug)]
pub struct GuessOptCfg<'a, T: OptValidator> {
    pub idx: usize,

    pub len: usize,

    pub arg: Option<ARef<RawVal>>,

    pub clopt: &'a CLOpt,

    pub opt_validator: &'a T,
}

impl<'a, T: OptValidator> GuessOptCfg<'a, T> {
    pub fn new(
        idx: usize,
        len: usize,
        arg: Option<ARef<RawVal>>,
        clopt: &'a CLOpt,
        opt_validator: &'a T,
    ) -> Self {
        Self {
            idx,
            len,
            arg,
            clopt,
            opt_validator,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn total(&self) -> usize {
        self.len
    }

    pub fn arg(&self) -> Option<&ARef<RawVal>> {
        self.arg.as_ref()
    }

    pub fn opt_validator(&self) -> &'a T {
        self.opt_validator
    }
}

#[derive(Debug)]
pub struct OptGuess<'a, S, T>(PhantomData<&'a (S, T)>);

impl<'a, S, T> Default for OptGuess<'a, S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S, T> OptGuess<'a, S, T> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }

    fn bool2str(value: bool) -> ARef<RawVal> {
        if value {
            RawVal::from(BOOL_TRUE).into()
        } else {
            RawVal::from(BOOL_FALSE).into()
        }
    }
}

impl<'a, S, T> Guess for OptGuess<'a, S, T>
where
    S: Set,
    T: OptValidator,
{
    type Config = GuessOptCfg<'a, T>;

    type Process = OptProcess<S>;

    fn guess(
        &mut self,
        style: &UserStyle,
        cfg: Self::Config,
    ) -> Result<Option<Self::Process>, Error> {
        let mut matches = vec![];
        let index = cfg.idx();
        let count = cfg.total();
        let clopt = &cfg.clopt;
        let mut any_match = false;

        match style {
            UserStyle::EqualWithValue => {
                if clopt.value.is_some() {
                    matches.push(
                        OptMatch::default()
                            .with_idx(index)
                            .with_total(count)
                            .with_arg(clopt.value.clone())
                            .with_style(Style::Argument)
                            .with_name(valueof("name", &clopt.name)?),
                    );
                }
            }
            UserStyle::Argument => {
                if clopt.value.is_none() && cfg.arg().is_some() {
                    matches.push(
                        OptMatch::default()
                            .with_idx(index)
                            .with_total(count)
                            .with_consume(true)
                            .with_arg(cfg.arg().cloned())
                            .with_style(Style::Argument)
                            .with_name(valueof("name", &clopt.name)?),
                    );
                }
            }
            UserStyle::EmbeddedValue => {
                if clopt.value.is_none() {
                    if let Some(name) = &clopt.name {
                        // strip the prefix before generate
                        let opt_validator = cfg.opt_validator();
                        let splited = opt_validator.split(name).map_err(Into::into)?;

                        // make sure we using `chars.count`, not len()
                        if splited.1.chars().count() >= 2 {
                            let prefix_len = splited.0.len() + 1;
                            let i = (prefix_len..name.len())
                                .find(|v| name.is_char_boundary(*v))
                                .unwrap();
                            let name_value = name.split_at(i);

                            matches.push(
                                OptMatch::default()
                                    .with_idx(index)
                                    .with_total(count)
                                    .with_arg(Some(RawVal::from(name_value.1).into()))
                                    .with_style(Style::Argument)
                                    .with_name(name_value.0.into()),
                            );
                        }
                    }
                }
            }
            UserStyle::EmbeddedValuePlus => {
                if clopt.value.is_none() {
                    if let Some(name) = &clopt.name {
                        let opt_validator = cfg.opt_validator();
                        let splited = opt_validator.split(name).map_err(Into::into)?;

                        // make sure we using `chars.count`, not len()
                        if splited.1.chars().count() >= 3 {
                            let mut i = splited.0.len() + 2;

                            while i < name.len() {
                                if let Some(next) =
                                    (i..name.len()).find(|v| name.is_char_boundary(*v))
                                {
                                    let name_value = name.split_at(next);

                                    matches.push(
                                        OptMatch::default()
                                            .with_idx(index)
                                            .with_total(count)
                                            .with_arg(Some(RawVal::from(name_value.1).into()))
                                            .with_style(Style::Argument)
                                            .with_name(name_value.0.into()),
                                    );
                                    any_match = true;
                                    i = next + 1;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            UserStyle::CombinedOption => {
                if clopt.value.is_none() {
                    if let Some(name) = &clopt.name {
                        let opt_validator = cfg.opt_validator();
                        let splited = opt_validator.split(name).map_err(Into::into)?;

                        if splited.1.chars().count() > 1 {
                            for char in splited.1.chars() {
                                matches.push(
                                    OptMatch::default()
                                        .with_idx(index)
                                        .with_total(count)
                                        .with_arg(Some(OptGuess::<S, T>::bool2str(true)))
                                        .with_style(Style::Combined)
                                        .with_name(format!("{}{}", splited.0, char).into()),
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
                            .with_total(count)
                            .with_arg(Some(OptGuess::<S, T>::bool2str(true)))
                            .with_style(Style::Boolean)
                            .with_name(valueof("name", &clopt.name)?),
                    );
                }
            }
            _ => {
                unimplemented!("Unsupport generate Process for NOA Style")
            }
        }

        Ok((!matches.is_empty()).then(|| {
            let mut process = Self::Process::new(matches);

            process.set_any_match(any_match);
            process
        }))
    }
}

/// Guess configuration for NOA.
pub struct GuessNOACfg {
    index: usize,
    total: usize,
    args: ARef<Args>,
}

impl GuessNOACfg {
    pub fn new(args: ARef<Args>, index: usize, total: usize) -> Self {
        Self { args, index, total }
    }

    pub fn idx(&self) -> usize {
        self.index
    }

    pub fn total(&self) -> usize {
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
        let count = cfg.total();
        let name = args.get(pos).and_then(|v| v.get_str()).map(Str::from);

        match style {
            UserStyle::Main => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_args(args)
                        .with_idx(pos)
                        .with_total(count)
                        .with_style(Style::Main)
                        .reset_arg(),
                );
            }
            UserStyle::Pos => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_args(args)
                        .with_idx(pos)
                        .with_total(count)
                        .with_style(Style::Pos)
                        .reset_arg(),
                );
            }
            UserStyle::Cmd => {
                mat = Some(
                    NOAMatch::default()
                        .with_name(name)
                        .with_args(args)
                        .with_idx(pos)
                        .with_total(count)
                        .with_style(Style::Cmd)
                        .reset_arg(),
                );
            }
            _ => {
                unimplemented!("Unsupport generate Process for Opt Style")
            }
        }

        Ok(mat.map(|v| Self::Process::new(Some(v))))
    }
}
