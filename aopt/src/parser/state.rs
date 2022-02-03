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
        let string = value.as_ref().ok_or(Self::gen_unwrap_error(name))?;
        Ok(string.clone())
    }

    pub fn gen_opt<M: Matcher>(&self, arg: &Argument) -> Result<Option<M>> {
        let mut ret: Vec<Box<dyn Context>> = vec![];

        match self {
            Self::PSEqualWithValue => {
                if arg.get_value().is_some() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        arg.get_value().clone(),
                        Style::Argument,
                        false,
                        arg.is_disabled(),
                    )));
                }
            }
            Self::PSArgument => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        arg.next.clone(),
                        Style::Argument,
                        true,
                        arg.is_disabled(),
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
                    )));
                }
            }
            Self::PSDelayEqualWithValue => {
                if arg.get_value().is_some() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        arg.get_value().clone(),
                        Style::Argument,
                        false,
                        arg.is_disabled(),
                    )));
                }
            }
            Self::PSDelayArgument => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        Self::do_unwrap("prefix", arg.get_prefix())?,
                        Self::do_unwrap("name", arg.get_name())?,
                        arg.next.clone(),
                        Style::Argument,
                        true,
                        arg.is_disabled(),
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
                    )));
                }
            }
            _ => {}
        }
        Ok(if ret.len() == 0 {
            None
        } else {
            let mut proc = M::default();

            for item in ret {
                proc.add_ctx(item);
            }

            Some(proc)
        })
    }

    pub fn gen_nonopt<M: Matcher>(
        &self,
        noa: &Ustr,
        total: u64,
        current: u64,
    ) -> Result<Option<M>> {
        let mut ret: Vec<Box<dyn Context>> = vec![];

        match self {
            Self::PSNonMain => {
                ret.push(Box::new(NonOptContext::new(
                    noa.clone(),
                    Style::Main,
                    total,
                    current,
                )));
            }
            Self::PSNonPos => {
                ret.push(Box::new(NonOptContext::new(
                    noa.clone(),
                    Style::Pos,
                    total,
                    current,
                )));
            }
            Self::PSNonCmd => {
                ret.push(Box::new(NonOptContext::new(
                    noa.clone(),
                    Style::Cmd,
                    total,
                    current,
                )));
            }
            _ => {}
        }
        Ok(if ret.len() == 0 {
            None
        } else {
            let mut proc = M::default();

            for item in ret {
                proc.add_ctx(item);
            }

            Some(proc)
        })
    }
}
