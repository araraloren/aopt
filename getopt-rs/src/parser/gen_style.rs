use crate::arg::Argument;
use crate::ctx::{Context, DelayContext, NonOptContext, OptContext};
use crate::opt::Style;
use crate::proc::Proc;

#[derive(Debug, Clone)]
pub enum GenStyle {
    GSEqualWithValue,

    GSArgument,

    GSEmbeddedValue,

    GSMultipleOption,

    GSBoolean,

    GSDelayEqualWithValue,

    GSDelayArgument,

    GSDelayEmbeddedValue,

    GSDelayMultipleOption,

    GSDelayBoolean,

    GSNonMain,

    GSNonPos,

    GSNonCmd,
}

impl<'pre> GenStyle {
    pub fn gen_opt<P: Proc + Default>(&self, arg: &Argument<'pre>) -> Option<P> {
        let mut ret: Vec<Box<dyn Context>> = vec![];

        match self {
            GenStyle::GSEqualWithValue => {
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
            GenStyle::GSArgument => {
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
            GenStyle::GSEmbeddedValue => {
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
            GenStyle::GSMultipleOption => {
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
            GenStyle::GSBoolean => {
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
            GenStyle::GSDelayEqualWithValue => {
                if arg.get_value().is_some() {
                    ret.push(Box::new(DelayContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        arg.get_value().clone(),
                        Style::Argument,
                        false,
                    )));
                }
            }
            GenStyle::GSDelayArgument => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(DelayContext::new(
                        arg.get_prefix().unwrap().clone(),
                        arg.get_name().as_ref().unwrap().clone(),
                        arg.next.clone(),
                        Style::Argument,
                        true,
                    )));
                }
            }
            GenStyle::GSDelayEmbeddedValue => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() >= 2 {
                            let name_value = name.split_at(1);

                            ret.push(Box::new(DelayContext::new(
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
            GenStyle::GSDelayMultipleOption => {
                if arg.get_value().is_none() {
                    if let Some(name) = arg.get_name() {
                        if name.len() > 1 {
                            for char in name.chars() {
                                ret.push(Box::new(DelayContext::new(
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
            GenStyle::GSDelayBoolean => {
                if arg.get_value().is_none() {
                    ret.push(Box::new(DelayContext::new(
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
            let mut proc = P::default();

            for item in ret {
                proc.add_ctx(item);
            }

            Some(proc)
        }
    }

    pub fn gen_nonopt<P: Proc + Default>(
        &self,
        noa: &String,
        total: u64,
        current: u64,
    ) -> Option<P> {
        let mut ret: Vec<Box<dyn Context>> = vec![];

        match self {
            GenStyle::GSNonMain => {
                ret.push(Box::new(NonOptContext::new(
                    noa.clone(),
                    Style::Main,
                    total,
                    current,
                )));
            }
            GenStyle::GSNonPos => {
                ret.push(Box::new(NonOptContext::new(
                    noa.clone(),
                    Style::Pos,
                    total,
                    current,
                )));
            }
            GenStyle::GSNonCmd => {
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
            let mut proc = P::default();

            for item in ret {
                proc.add_ctx(item);
            }

            Some(proc)
        }
    }
}
