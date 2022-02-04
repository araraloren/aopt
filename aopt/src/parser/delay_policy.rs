use super::ParserState;
use std::fmt::Debug;

use super::Policy;
use super::Service;
use crate::arg::Argument;
use crate::err::Error;
use crate::err::Result;
use crate::parser::ValueKeeper;
use crate::proc::Matcher;
use crate::proc::NonOptMatcher;
use crate::proc::OptMatcher;
use crate::set::OptionInfo;
use crate::set::Set;
use ustr::Ustr;

#[derive(Debug, Clone, Default)]
pub struct DelayPolicy {
    strict: bool,
    value_keeper: Vec<ValueKeeper>,
}

impl DelayPolicy {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    pub fn add_delay_value(&mut self, value: ValueKeeper) {
        self.value_keeper.push(value);
    }
}

impl<S: Set, SS: Service> Policy<S, SS> for DelayPolicy {
    fn parse(
        &mut self,
        set: &mut S,
        service: &mut SS,
        iter: &mut dyn Iterator<Item = Argument>,
    ) -> Result<bool> {
        // copy the prefix, so we don't need borrow set
        let prefix: Vec<Ustr> = set.get_prefix().iter().map(|v| v.clone()).collect();

        // add info to Service
        for opt in set.opt_iter() {
            service
                .get_subscriber_info_mut()
                .push(Box::new(OptionInfo::from(opt)));
        }

        // reset set and do pre check
        info!("reset and do pre check");
        set.reset();
        service.pre_check(set)?;

        let parser_state = vec![
            ParserState::PSDelayEqualWithValue,
            ParserState::PSDelayArgument,
            ParserState::PSDelayBoolean,
            ParserState::PSDelayMultipleOption,
            ParserState::PSDelayEmbeddedValue,
        ];

        // iterate the Arguments, generate option context
        // send it to Publisher
        info!("start process option ...");
        while let Some(mut arg) = iter.next() {
            let mut matched = false;
            let mut consume = false;

            debug!(?arg, "iterator Argument ...");
            if let Ok(ret) = arg.parse(&prefix) {
                if ret {
                    debug!(?arg, "after parsing ...");
                    for gen_style in &parser_state {
                        if let Some(mut proc) = service.gen_opt::<OptMatcher>(&arg, &gen_style)? {
                            let value_keeper = service.matching(&mut proc, set, false)?;

                            if proc.is_matched() {
                                for value in value_keeper {
                                    self.add_delay_value(value);
                                }
                                matched = true;
                            }
                            if proc.is_comsume_argument() {
                                consume = true;
                            }
                            if matched {
                                break;
                            }
                        }
                    }
                    if !matched {
                        // if current ARG is like an option, but it not matched
                        if self.strict {
                            return Err(Error::sp_invalid_option_name(
                                arg.current.unwrap_or_default().as_ref(),
                            ));
                        }
                    }
                }
            }
            if matched && consume {
                iter.next();
            } else if !matched {
                debug!("!!! {:?} not matching, will add it to noa", &arg);
                if let Some(noa) = &arg.current {
                    service.get_noa_mut().push(noa.clone());
                }
            }
        }

        let noa = service.get_noa().clone();

        trace!(?noa, "current non-option argument");
        let noa_count = noa.len();

        if noa_count > 0 {
            let gen_style = ParserState::PSNonCmd;

            info!("start process {:?} ...", &gen_style);
            if let Some(mut proc) =
                service.gen_nonopt::<NonOptMatcher>(&noa[0], noa_count, 1, &gen_style)?
            {
                service.matching(&mut proc, set, true)?;
            }

            let gen_style = ParserState::PSNonPos;

            info!("start process {:?} ...", &gen_style);
            for index in 1..=noa_count {
                if let Some(mut proc) = service.gen_nonopt::<NonOptMatcher>(
                    &noa[index - 1],
                    noa_count,
                    index,
                    &gen_style,
                )? {
                    service.matching(&mut proc, set, true)?;
                }
            }
        }

        for ValueKeeper { id, index, value } in std::mem::take(&mut self.value_keeper) {
            let ret_value = if service.get_callback().contains_key(&id) {
                service.invoke(id, set, index, value)?
            } else {
                Some(value)
            };
            set[id].as_mut().set_callback_ret(ret_value)?;
        }

        info!("do opt check");
        service.opt_check(set)?;

        // check pos and cmd
        info!("do nonopt check");
        service.nonopt_check(set)?;

        let gen_style = ParserState::PSNonMain;

        info!("start process {:?} ...", &gen_style);
        if let Some(mut proc) =
            service.gen_nonopt::<NonOptMatcher>(&Ustr::default(), noa_count, 1, &gen_style)?
        {
            service.matching(&mut proc, set, true)?;
        }

        // do post check
        service.post_check(set)?;

        Ok(true)
    }
}
