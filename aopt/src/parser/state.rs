use std::ops::Not;

use crate::arg::Argument;
use crate::ctx::Context;
use crate::ctx::NonOptContext;
use crate::ctx::OptContext;
use crate::err::Error;
use crate::err::Result;
use crate::opt::Style;
use crate::proc::Matcher;
use ustr::Ustr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserState {
    PSEqualWithValue,

    PSArgument,

    PSEmbeddedValue,

    PSMultipleOption,

    PSBoolean,

    PSDelayEqualWithValue,

    PSDelayArgument,

    PSDelayEmbeddedValue,

    PSDelayMultipleOption,

    PSDelayBoolean,

    PSNonMain,

    PSNonPos,

    PSNonCmd,

    PSPreCheck,

    PSPostCheck,

    PSOptCheck,

    PSNonOptCheck,

    PSCustom(u64),
}

impl ParserState {
    fn gen_unwrap_error(name: &str) -> Error {
        Error::arg_unwrap_value_failed(name)
    }

    fn do_unwrap(name: &str, value: &Option<Ustr>) -> Result<Ustr> {
        let string = value.as_ref().ok_or_else(|| Self::gen_unwrap_error(name))?;
        Ok(*string)
    }

    pub fn gen_opt<M: Matcher + Default>(
        &self,
        arg: &Argument,
        index: usize,
        total: usize,
    ) -> Result<Option<M>> {
        let mut ret: Vec<Box<dyn Context>> = vec![];

        match self {
            Self::PSEqualWithValue => {
                if arg.get_value().is_some() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        *arg.get_value(),
                        Style::Argument,
                        false,
                        arg.is_disabled(),
                        index,
                        total,
                    )));
                }
            }
            Self::PSArgument => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        arg.next,
                        Style::Argument,
                        true,
                        arg.is_disabled(),
                        index,
                        total,
                    )));
                }
            }
            Self::PSEmbeddedValue => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() >= 2 {
                            let name_value = name.split_at(1);

                            ret.push(Box::new(OptContext::new(
                                Self::do_unwrap("prefix", arg.get_prefix())?,
                                name_value.0.into(),
                                Some(name_value.1.into()),
                                Style::Argument,
                                false,
                                arg.is_disabled(),
                                index,
                                total,
                            )));
                        }
                    }
                }
            }
            Self::PSMultipleOption => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() > 1 {
                            for char in name.chars() {
                                ret.push(Box::new(OptContext::new(
                                    Self::do_unwrap("prefix", arg.get_prefix())?,
                                    format!("{}", char).into(),
                                    None,
                                    Style::Multiple,
                                    false,
                                    arg.is_disabled(),
                                    index,
                                    total,
                                )));
                            }
                        }
                    }
                }
            }
            Self::PSBoolean => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        None,
                        Style::Boolean,
                        false,
                        arg.is_disabled(),
                        index,
                        total,
                    )));
                }
            }
            Self::PSDelayEqualWithValue => {
                if arg.get_value().is_some() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        *arg.get_value(),
                        Style::Argument,
                        false,
                        arg.is_disabled(),
                        index,
                        total,
                    )));
                }
            }
            Self::PSDelayArgument => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        arg.next,
                        Style::Argument,
                        true,
                        arg.is_disabled(),
                        index,
                        total,
                    )));
                }
            }
            Self::PSDelayEmbeddedValue => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() >= 2 {
                            let name_value = name.split_at(1);

                            ret.push(Box::new(OptContext::new(
                                Self::do_unwrap("prefix", arg.get_prefix())?,
                                name_value.0.into(),
                                Some(name_value.1.into()),
                                Style::Argument,
                                false,
                                arg.is_disabled(),
                                index,
                                total,
                            )));
                        }
                    }
                }
            }
            Self::PSDelayMultipleOption => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() > 1 {
                            for char in name.chars() {
                                ret.push(Box::new(OptContext::new(
                                    Self::do_unwrap("prefix", arg.get_prefix())?,
                                    format!("{}", char).into(),
                                    None,
                                    Style::Multiple,
                                    false,
                                    arg.is_disabled(),
                                    index,
                                    total,
                                )));
                            }
                        }
                    }
                }
            }
            Self::PSDelayBoolean => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        None,
                        Style::Boolean,
                        false,
                        arg.is_disabled(),
                        index,
                        total,
                    )));
                }
            }
            _ => {}
        }
        Ok(ret.is_empty().not().then(|| {
            let mut proc = M::default();

            for item in ret {
                proc.add_ctx(item);
            }

            proc
        }))
    }

    pub fn gen_nonopt<M: Matcher + Default>(
        &self,
        noa: &Ustr,
        index: usize,
        total: usize,
    ) -> Result<Option<M>> {
        let mut ret: Vec<Box<dyn Context>> = vec![];

        match self {
            Self::PSNonMain => {
                ret.push(Box::new(NonOptContext::new(
                    *noa,
                    Style::Main,
                    index,
                    total,
                )));
            }
            Self::PSNonPos => {
                ret.push(Box::new(NonOptContext::new(*noa, Style::Pos, index, total)));
            }
            Self::PSNonCmd => {
                ret.push(Box::new(NonOptContext::new(*noa, Style::Cmd, index, total)));
            }
            _ => {}
        }
        Ok(ret.is_empty().not().then(|| {
            let mut proc = M::default();

            for item in ret {
                proc.add_ctx(item);
            }

            proc
        }))
    }
}
