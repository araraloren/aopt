use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;

use super::state::ParserState;
use super::Service;
use super::ValueKeeper;
use crate::opt::OptCallback;
use crate::opt::OptValue;
use crate::opt::Style;
use crate::proc::Info;
use crate::proc::Matcher;
use crate::proc::Matching;
use crate::set::Set;
use crate::uid::Uid;
use crate::Error;
use crate::Result;
use ustr::Ustr;

#[derive(Debug, Default)]
pub struct DefaultService {
    noa: Vec<Ustr>,

    subscriber_info: Vec<Box<dyn Info>>,

    callback_info: HashMap<Uid, RefCell<OptCallback>>,
}

impl DefaultService {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn register<I: 'static + Info>(&mut self, info: I) -> &mut Self {
        self.subscriber_info.push(Box::new(info));
        self
    }
}

impl Service for DefaultService {
    fn gen_opt<M: Matcher>(
        &self,
        arg: &crate::arg::Argument,
        style: &ParserState,
    ) -> Result<Option<M>> {
        Ok(style.gen_opt(arg)?)
    }

    fn gen_nonopt<M: Matcher>(
        &self,
        noa: &ustr::Ustr,
        total: usize,
        current: usize,
        style: &ParserState,
    ) -> Result<Option<M>> {
        Ok(style.gen_nonopt(noa, total as u64, current as u64)?)
    }

    fn process_opt<M: Matcher, S: Set>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        invoke: bool,
    ) -> Result<Vec<ValueKeeper>> {
        Ok(self.matching_opt(matcher, set, invoke)?)
    }

    fn process_nonopt<M: Matcher, S: Set>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        invoke: bool,
    ) -> Result<Vec<ValueKeeper>> {
        Ok(self.matching_nonopt(matcher, set, invoke)?)
    }

    fn pre_check<S: Set>(&self, set: &S) -> Result<bool> {
        for (uid, callback) in &self.callback_info {
            if let Some(opt) = set.get_opt(*uid) {
                if !opt.is_accept_callback_type(callback.borrow().to_callback_type()) {
                    return Err(Error::opt_unsupport_callback_type(
                        opt.get_hint().as_ref(),
                        &format!("{:?}", callback.borrow().to_callback_type()),
                    ));
                }
            } else {
                warn!(%uid, "callback has unknow option uid");
            }
        }
        Ok(true)
    }

    fn opt_check<S: Set>(&self, set: &S) -> Result<bool> {
        for opt in set.opt_iter() {
            if opt.as_ref().match_style(Style::Boolean)
                || opt.as_ref().match_style(Style::Argument)
                || opt.as_ref().match_style(Style::Multiple)
            {
                opt.check()?;
            }
        }
        Ok(true)
    }

    fn nonopt_check<S: Set>(&self, set: &S) -> Result<bool> {
        const MAX_INDEX: u64 = u64::MAX;

        let mut index_map: HashMap<u64, Vec<Uid>> = HashMap::new();

        for opt in set.opt_iter() {
            if opt.as_ref().match_style(Style::Pos)
                || opt.as_ref().match_style(Style::Cmd)
                || opt.as_ref().match_style(Style::Main)
            {
                if let Some(index) = opt.as_ref().get_index() {
                    let index = index.calc_index(MAX_INDEX, 1).unwrap_or(MAX_INDEX);
                    let entry = index_map.entry(index).or_insert(vec![]);

                    entry.push(opt.as_ref().get_uid());
                }
            }
        }

        trace!(?index_map, "non-opt check information");

        let mut names = vec![];

        for (index, uids) in index_map.iter() {
            let valid;

            // <cmd1> <cmd2> <pos3> [pos4] [pos5]
            // any of thing at position 1
            if index == &1 || index == &0 {
                let mut cmd_count = 0;
                let mut cmd_valid = false;
                let mut pos_valid = true;
                let mut force_valid = false;

                for uid in uids {
                    let opt = set.get_opt(*uid).unwrap();

                    if opt.match_style(Style::Cmd) {
                        cmd_count += 1;
                        // set the cmd will valid the check
                        // if any of cmd is valid, break out
                        cmd_valid = cmd_valid || opt.check().is_ok();
                        if cmd_valid {
                            break;
                        }
                        names.push(opt.get_hint().to_owned());
                    } else if opt.match_style(Style::Pos) {
                        let opt_valid = opt.check().is_ok();

                        pos_valid = pos_valid && opt_valid;
                        if opt_valid && !opt.get_optional() {
                            force_valid = true;
                            names.push(opt.get_hint().to_owned());
                        }
                    }
                }

                debug!(%cmd_valid, %pos_valid, %force_valid, "in default nonopt-check");

                // if we have CMD, then the CMD must be set or any POS is set
                // if all nonopt @1 are POS, it's normally like @2..
                if cmd_count > 0 {
                    valid = cmd_valid || (pos_valid && force_valid);
                } else {
                    valid = pos_valid;
                }
            } else {
                // <pos1> [pos2] [pos3] [pos4] [pos5]
                // if any of POS is force required, then it must set by user
                let mut pos_valid = true;

                for uid in uids {
                    let opt = set.get_opt(*uid).unwrap();
                    let opt_valid = opt.check().is_ok();

                    pos_valid = pos_valid && opt_valid;
                    if !opt_valid {
                        names.push(opt.get_hint().to_owned());
                    }
                }
                debug!(%pos_valid, "in default nonopt-check");
                valid = pos_valid;
            }
            if !valid {
                return Err(Error::sp_pos_force_require(*index, names.join(" | ")));
            }
            names.clear();
        }

        Ok(true)
    }

    fn post_check<S: Set>(&self, _set: &S) -> Result<bool> {
        Ok(true)
    }

    fn invoke<S: Set>(
        &self,
        uid: Uid,
        set: &mut S,
        noa_idx: usize,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        if let Some(callback) = self.callback_info.get(&uid) {
            debug!("calling callback of option<{}>", uid);
            match callback.borrow_mut().deref_mut() {
                OptCallback::Opt(cb) => cb.as_mut().call(uid, set, value),
                OptCallback::OptMut(cb) => cb.as_mut().call(uid, set, value),
                OptCallback::Pos(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_idx - 1], noa_idx as u64, value)
                }
                OptCallback::PosMut(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_idx - 1], noa_idx as u64, value)
                }
                OptCallback::Main(cb) => {
                    let noaref: Vec<&str> = self.noa.iter().map(|v| v.as_ref()).collect();
                    cb.as_mut().call(uid, set, &noaref, value)
                }
                OptCallback::MainMut(cb) => {
                    let noaref: Vec<&str> = self.noa.iter().map(|v| v.as_ref()).collect();
                    cb.as_mut().call(uid, set, &noaref, value)
                }
                OptCallback::Null => Ok(None),
            }
        } else {
            Ok(Some(value))
        }
    }

    fn get_callback(&self) -> &HashMap<Uid, RefCell<OptCallback>> {
        &self.callback_info
    }

    fn get_subscriber_info<I: 'static + Info>(&self) -> &Vec<Box<dyn Info>> {
        &self.subscriber_info
    }

    fn get_noa(&self) -> &Vec<Ustr> {
        &self.noa
    }

    fn get_callback_mut(&mut self) -> &mut HashMap<Uid, RefCell<OptCallback>> {
        &mut self.callback_info
    }

    fn get_subscriber_info_mut(&mut self) -> &mut Vec<Box<dyn Info>> {
        &mut self.subscriber_info
    }

    fn get_noa_mut(&mut self) -> &mut Vec<Ustr> {
        &mut self.noa
    }

    fn reset(&mut self) {
        self.subscriber_info.clear();
        self.callback_info.clear();
        self.noa.clear();
    }
}

impl<M: Matcher> Matching<M> for DefaultService {
    fn matching_nonopt<S: Set>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        _invoke: bool,
    ) -> Result<Vec<ValueKeeper>> {
        let mut matched = true;

        debug!(?matcher, "process matcher in nonopt way: ");
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set).unwrap_or(None);

            if let Some(ctx) = ctx {
                if ctx.is_matched() {
                    let opt = set[uid].as_mut();
                    let invoke_callback = opt.is_need_invoke();
                    let mut value = ctx.take_value();

                    assert_eq!(value.is_some(), true);
                    if invoke_callback {
                        let has_callback = self.get_callback().contains_key(&uid);

                        if has_callback {
                            // invoke callback of current option/non-option
                            // make matched true, if any of non-option callback return Some(*)
                            value = self.invoke(
                                uid,
                                set,
                                ctx.get_matched_index().unwrap_or_default(),
                                value.unwrap(),
                            )?;
                            if value.is_none() {
                                // Ok(None) treat as user said current non-option not matched
                                matched = true;
                            }
                        }
                        // reborrow the opt avoid the compiler error
                        // reset the matcher, we need match all the non-option
                        debug!(?value, "get callback return value");
                        set[uid].as_mut().set_invoke(false);
                        matcher.reset();
                    }

                    // set the value after invoke
                    set[uid].as_mut().set_callback_ret(value)?;
                }
            }
        }
        if !matched {
            matcher.undo(set);
        }
        Ok(vec![])
    }

    fn matching_opt<S: Set>(
        &mut self,
        msg: &mut M,
        set: &mut S,
        invoke: bool,
    ) -> Result<Vec<ValueKeeper>> {
        let matcher = msg;
        let mut value_keeper: Vec<ValueKeeper> = vec![];

        debug!(?matcher, "process matcher in opt way: ");
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set).unwrap_or(None);

            if let Some(ctx) = ctx {
                if ctx.is_matched() {
                    let opt = set[uid].as_mut();
                    let invoke_callback = opt.is_need_invoke();
                    let value = ctx.take_value();

                    assert_eq!(value.is_some(), true);
                    if invoke_callback {
                        opt.set_invoke(false);
                    }

                    // add the value to value keeper, call the callback after cmd/pos processed
                    info!("add {:?} to delay parser value keeper", &uid);
                    value_keeper.push(ValueKeeper {
                        id: uid,
                        index: ctx.get_matched_index().unwrap_or_default(),
                        value: value.unwrap(),
                    });
                }
            }
        }
        if matcher.is_matched() && invoke {
            // do value set and invoke callback
            for ValueKeeper { id, index, value } in value_keeper {
                let ret_value = if self.get_callback().contains_key(&id) {
                    self.invoke(id, set, index, value)?
                } else {
                    Some(value)
                };
                set[id].as_mut().set_callback_ret(ret_value)?;
            }
            return Ok(vec![]);
        }
        if !matcher.is_matched() {
            matcher.undo(set);
        }
        Ok(value_keeper)
    }
}
