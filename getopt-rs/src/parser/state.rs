use crate::arg::Argument;
use crate::ctx::{Context, NonOptContext, OptContext};
use crate::opt::Style;
use crate::proc::Matcher;

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

impl<'pre> ParserState {
    pub fn gen_opt<M: Matcher + Default>(&self, arg: &Argument<'pre>) -> Option<M> {
        let mut ret: Vec<Box<dyn Context>> = vec![];

        match self {
            Self::PSEqualWithValue => {
                if arg.get_value().is_some() {
                    ret.push(Box::new(OptContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        arg.get_value().clone(),
                        Style::Argument,
                        false,
                    )));
                }
            }
            Self::PSArgument => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        arg.next.clone(),
                        Style::Argument,
                        true,
                    )));
                }
            }
            Self::PSEmbeddedValue => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() >= 2 {
                            let name_value = name.split_at(1);

                            ret.push(Box::new(OptContext::new(
                                arg.get_prefix().unwrap().clone(),
                                name_value.0.to_owned(),
                                Some(name_value.1.to_owned()),
                                Style::Argument,
                                false,
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
                                    arg.get_prefix().unwrap().clone(),
                                    String::from(char),
                                    None,
                                    Style::Multiple,
                                    false,
                                )));
                            }
                        }
                    }
                }
            }
            Self::PSBoolean => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        None,
                        Style::Boolean,
                        false,
                    )));
                }
            }
            Self::PSDelayEqualWithValue => {
                if arg.get_value().is_some() {
                    ret.push(Box::new(OptContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        arg.get_value().clone(),
                        Style::Argument,
                        false,
                    )));
                }
            }
            Self::PSDelayArgument => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        arg.next.clone(),
                        Style::Argument,
                        true,
                    )));
                }
            }
            Self::PSDelayEmbeddedValue => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() >= 2 {
                            let name_value = name.split_at(1);

                            ret.push(Box::new(OptContext::new(
                                arg.get_prefix().unwrap().clone(),
                                name_value.0.to_owned(),
                                Some(name_value.1.to_owned()),
                                Style::Argument,
                                false,
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
                                    arg.get_prefix().unwrap().clone(),
                                    String::from(char),
                                    None,
                                    Style::Multiple,
                                    false,
                                )));
                            }
                        }
                    }
                }
            }
            Self::PSDelayBoolean => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(OptContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        None,
                        Style::Boolean,
                        false,
                    )));
                }
            }
            _ => {}
        }
        if ret.len() == 0 {
            None
        } else {
            let mut proc = M::default();

            for item in ret {
                proc.add_ctx(item);
            }

            Some(proc)
        }
    }

    pub fn gen_nonopt<M: Matcher + Default>(
        &self,
        noa: &String,
        total: u64,
        current: u64,
    ) -> Option<M> {
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
        if ret.len() == 0 {
            None
        } else {
            let mut proc = M::default();

            for item in ret {
                proc.add_ctx(item);
            }

            Some(proc)
        }
    }
}
