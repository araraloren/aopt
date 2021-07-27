use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;

use super::HashMapIter;
use super::ParserState;
use super::{Parser, ReturnValue};
use crate::arg::ArgStream;
use crate::err::Result;
use crate::opt::{OptCallback, OptValue, Style};
use crate::proc::{Info, Matcher, NonOptMatcher, OptMatcher, Proc};
use crate::set::{OptionInfo, Set};
use crate::uid::{Generator, Uid};

#[derive(Debug, Default)]
pub struct PreParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    uid_gen: G,

    subscriber_info: Vec<Box<dyn Info>>,

    callback: HashMap<Uid, RefCell<OptCallback>>,

    noa: Vec<String>,

    marker: PhantomData<S>,
}

impl<S, G> PreParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    pub fn new(uid_gen: G) -> Self {
        Self {
            uid_gen,
            ..Self::default()
        }
    }
}

impl<S, G> Parser<S> for PreParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>> {
        let mut argstream = ArgStream::from(iter);
        let mut set = set;
        let mut iter = argstream.iter_mut();

        // copy the prefix, so we don't need borrow set
        let prefix: Vec<String> = set.get_prefix().iter().map(|v| v.clone()).collect();

        // add info to Proc
        for opt in set.iter() {
            self.subscriber_info
                .push(Box::new(OptionInfo::from(opt.get_uid())));
        }

        // reset set and do pre check
        set.reset();
        self.pre_check(&set)?;

        let parser_state = vec![
            ParserState::PSEqualWithValue,
            ParserState::PSArgument,
            ParserState::PSBoolean,
            ParserState::PSMultipleOption,
            ParserState::PSEmbeddedValue,
        ];

        // iterate the Arguments, generate option context
        // send it to Publisher
        debug!("Start process option ...");
        while let Some(arg) = iter.next() {
            let mut matched = false;
            let mut consume = false;

            debug!("Get next Argument => {:?}", &arg);
            if let Ok(ret) = arg.parse(&prefix) {
                if ret {
                    debug!(" ... parsed: {:?}", &arg);
                    for gen_style in &parser_state {
                        if let Some(ret) = gen_style.gen_opt::<OptMatcher>(arg) {
                            let mut proc = ret;

                            if self.process(&mut proc, &mut set)? {
                                if proc.is_matched() {
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
                    }
                }
            }
            if matched && consume {
                iter.next();
            } else if !matched {
                debug!("!!! Not matching {:?}, add it to noa", &arg);
                if let Some(noa) = &arg.current {
                    self.noa.push(noa.clone());
                }
            }
        }

        // do option check
        self.check_opt(&set)?;

        let noa_count = self.noa.len();

        if noa_count > 0 {
            let gen_style = ParserState::PSNonCmd;

            debug!("Start process {:?} ...", &gen_style);
            if let Some(ret) =
                gen_style.gen_nonopt::<NonOptMatcher>(&self.noa[0], noa_count as u64, 1)
            {
                let mut proc = ret;

                self.process(&mut proc, &mut set)?;
            }

            let gen_style = ParserState::PSNonPos;

            debug!("Start process {:?} ...", &gen_style);
            for index in 1..=noa_count {
                if let Some(ret) = gen_style.gen_nonopt::<NonOptMatcher>(
                    &self.noa[index - 1],
                    noa_count as u64,
                    index as u64,
                ) {
                    let mut proc = ret;

                    self.process(&mut proc, &mut set)?;
                }
            }
        }

        // check pos and cmd
        self.check_nonopt(&set)?;

        let gen_style = ParserState::PSNonMain;

        debug!("Start process {:?} ...", &gen_style);
        if let Some(ret) =
            gen_style.gen_nonopt::<NonOptMatcher>(&String::new(), noa_count as u64, 1)
        {
            let mut proc = ret;

            self.process(&mut proc, &mut set)?;
        }

        // do post check
        self.post_check(&set)?;

        Ok(Some(ReturnValue {
            set: set,
            noa: &self.noa,
        }))
    }

    fn invoke_callback(
        &self,
        uid: Uid,
        set: &mut S,
        noa_index: usize,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        if let Some(callback) = self.callback.get(&uid) {
            debug!("calling callback of option<{}>", uid);
            match callback.borrow_mut().deref_mut() {
                OptCallback::Opt(cb) => cb.as_mut().call(uid, set, value),
                OptCallback::OptMut(cb) => cb.as_mut().call(uid, set, value),
                OptCallback::Pos(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_index - 1], noa_index as u64, value)
                }
                OptCallback::PosMut(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_index - 1], noa_index as u64, value)
                }
                OptCallback::Main(cb) => cb.as_mut().call(uid, set, &self.noa, value),
                OptCallback::MainMut(cb) => cb.as_mut().call(uid, set, &self.noa, value),
                OptCallback::Null => Ok(None),
            }
        } else {
            Ok(Some(value))
        }
    }

    fn add_callback(&mut self, uid: Uid, callback: OptCallback) {
        self.callback.insert(uid, RefCell::new(callback));
    }

    fn get_callback(&self, uid: Uid) -> Option<&RefCell<OptCallback>> {
        self.callback.get(&uid)
    }

    fn callback_iter(&self) -> HashMapIter<'_, Uid, RefCell<OptCallback>> {
        self.callback.iter()
    }

    fn reset(&mut self) {
        self.uid_gen.reset();
        self.noa.clear();
        self.subscriber_info.clear();
    }
}

impl<S, G> Proc<S, NonOptMatcher> for PreParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    fn process(&mut self, msg: &mut NonOptMatcher, set: &mut S) -> Result<bool> {
        let matcher = msg;
        let mut matched = false;

        debug!("Got message<{}>: {:?}", &matcher.uid(), &matcher);
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set).unwrap_or(None);

            if let Some(ctx) = ctx {
                let opt = set[uid].as_mut();

                if let Some(noa_index) = ctx.get_matched_index() {
                    let invoke_callback = opt.is_need_invoke();
                    let mut value = ctx.take_value();

                    assert_eq!(value.is_some(), true);
                    if invoke_callback {
                        let has_callback = self.get_callback(uid).is_some();

                        if has_callback {
                            // invoke callback of current option/non-option
                            value = self.invoke_callback(uid, set, noa_index, value.unwrap())?;
                            if value.is_some() {
                                // make matched true, if any of NonOpt callback return Some(*)
                                matched = true;
                            }
                        } else {
                            // if a Cmd is matched, then the M matched
                            if opt.match_style(Style::Cmd) {
                                matched = true;
                            }
                        }
                        // reborrow the opt avoid the compiler error
                        debug!("In Proc, get return value of option<{}> = {:?}", uid, value);
                        set[uid].as_mut().set_invoke(false);
                        // reset the matcher, we need match all the NonOpt
                        matcher.reset();
                    }

                    // set the value after invoke
                    set[uid].as_mut().set_callback_ret(value)?;
                }
            }
        }
        Ok(matched)
    }
}

impl<S, G> Proc<S, OptMatcher> for PreParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    fn process(&mut self, msg: &mut OptMatcher, set: &mut S) -> Result<bool> {
        let matcher = msg;

        debug!("Got message<{}>: {:?}", &matcher.uid(), &matcher);
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set).unwrap_or(None);

            if let Some(ctx) = ctx {
                let opt = set[uid].as_mut();

                if let Some(noa_index) = ctx.get_matched_index() {
                    let invoke_callback = opt.is_need_invoke();
                    let mut value = ctx.take_value();

                    assert_eq!(value.is_some(), true);

                    if invoke_callback {
                        // invoke callback of current option/non-option
                        value = self.invoke_callback(uid, set, noa_index, value.unwrap())?;

                        debug!("Get return value of option<{}> = {:?}", uid, value);
                        set[uid].as_mut().set_invoke(false);
                    }

                    set[uid].as_mut().set_callback_ret(value)?;
                }
            }
        }
        Ok(matcher.is_matched())
    }
}