use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

use super::state::ParserState;
use super::Service;
use super::ValueKeeper;
use crate::opt::OptCallback;
use crate::opt::OptIndex;
use crate::opt::OptValue;
use crate::opt::Style;
use crate::proc::Info;
use crate::proc::Matcher;
use crate::proc::Proc;
use crate::set::Set;
use crate::uid::Uid;
use crate::Error;
use crate::Result;
use ustr::Ustr;

/// Simple wrapper of `HashMap<Uid, OptCallback>`.
#[derive(Debug, Default)]
pub struct CallbackStore<S: Set>(pub HashMap<Uid, OptCallback<S>>);

impl<S: Set> Deref for CallbackStore<S> {
    type Target = HashMap<Uid, OptCallback<S>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Set> DerefMut for CallbackStore<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> CallbackStore<S> {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    pub fn add_callback(&mut self, uid: Uid, cb: OptCallback<S>) {
        self.0.insert(uid, cb);
    }

    pub fn get_callback(&self, uid: &Uid) -> Option<&OptCallback<S>> {
        self.0.get(uid)
    }

    pub fn get_callback_mut(&mut self, uid: &Uid) -> Option<&mut OptCallback<S>> {
        self.0.get_mut(uid)
    }

    pub fn for_each(&self, f: impl Fn(&Uid, &OptCallback<S>) -> Result<bool>) -> Result<bool> {
        for (uid, cb) in self.0.iter() {
            f(uid, cb)?;
        }
        Ok(true)
    }
}

/// Simple implementation of [`Service`].
#[derive(Debug)]
pub struct SimpleService<S: Set> {
    noa: Vec<Ustr>,

    subscriber_info: Vec<Box<dyn Info>>,

    callback_store: CallbackStore<S>,
}

impl<S: Set + Default> Default for SimpleService<S> {
    fn default() -> Self {
        Self {
            callback_store: CallbackStore::new(),
            noa: Vec::default(),
            subscriber_info: Vec::default(),
        }
    }
}

impl<S: Set> SimpleService<S> {
    pub fn new() -> Self {
        Self {
            callback_store: CallbackStore::new(),
            noa: Vec::default(),
            subscriber_info: Vec::default(),
        }
    }

    pub fn register<I: 'static + Info>(&mut self, info: I) -> &mut Self {
        self.subscriber_info.push(Box::new(info));
        self
    }

    pub fn matching_nonopt<M: Matcher>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        _invoke: bool,
    ) -> Result<Vec<ValueKeeper>> {
        let mut matched = true;
        let subscriber_infos: Vec<Uid> =
            self.subscriber_info.iter().map(|v| v.info_uid()).collect();

        debug!(?matcher, "process matcher in nonopt way: ");
        for uid in subscriber_infos {
            let ctx = matcher.process(uid, set).unwrap_or(None);

            if let Some(ctx) = ctx {
                if ctx.is_matched() {
                    let opt = set.get_opt(uid).unwrap();
                    let invoke_callback = opt.is_need_invoke();
                    let mut value = ctx.take_value();

                    assert!(value.is_some());
                    if invoke_callback {
                        let has_callback = self.get_callback().contains_key(&uid);

                        if has_callback {
                            // invoke callback of current Opt/NonOpt
                            // make matched true, if any of NonOpt callback return Some(*)
                            value = self.invoke(
                                uid,
                                set,
                                ctx.get_matched_index().unwrap_or_default(),
                                value.unwrap(),
                            )?;
                            if value.is_none() {
                                // Ok(None) treat as user said current NonOpt not matched
                                matched = true;
                            }
                        }
                        // reborrow the opt avoid the compiler error
                        // reset the matcher, we need match all the NonOpt
                        debug!(?value, "get callback return value");
                        set.get_opt_mut(uid).unwrap().set_invoke(false);
                        matcher.reset();
                    }

                    // set the value after invoke
                    set.get_opt_mut(uid).unwrap().set_callback_ret(value)?;
                }
            }
        }
        if !matched {
            matcher.undo(set);
        }
        Ok(vec![])
    }

    pub fn matching_opt<M: Matcher>(
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
                    // just unwrap it, the uid must exists
                    let opt = set.get_opt_mut(uid).unwrap();
                    let invoke_callback = opt.is_need_invoke();
                    let value = ctx.take_value();

                    assert!(value.is_some());
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
                set.get_opt_mut(id).unwrap().set_callback_ret(ret_value)?;
            }
            return Ok(vec![]);
        }
        if !matcher.is_matched() {
            matcher.undo(set);
        }
        Ok(value_keeper)
    }
}

impl<S: Set> Service<S> for SimpleService<S> {
    fn gen_opt<M: Matcher + Default>(
        &self,
        arg: &crate::arg::Argument,
        style: &ParserState,
        arg_index: u64,
    ) -> Result<Option<M>> {
        style.gen_opt(arg, arg_index)
    }

    fn gen_nonopt<M: Matcher + Default>(
        &self,
        noa: &ustr::Ustr,
        total: usize,
        current: usize,
        style: &ParserState,
    ) -> Result<Option<M>> {
        style.gen_nonopt(noa, total as u64, current as u64)
    }

    fn matching<M: Matcher + Default>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        invoke: bool,
    ) -> Result<Vec<ValueKeeper>> {
        self.process(matcher, set, invoke)
    }

    /// Check the [Callback](crate::opt::OptCallback)'s type is matched with option [`CallbackType`](crate::opt::CallbackType).
    /// Check CMD and force required POS@1 are not exists same time.
    fn pre_check(&self, set: &S) -> Result<bool> {
        self.callback_store.for_each(|uid, cb| {
            if let Some(opt) = set.get_opt(*uid) {
                if !opt.is_accept_callback_type(cb.to_callback_type()) {
                    Err(Error::opt_unsupport_callback_type(
                        opt.get_hint().as_ref(),
                        &format!("{:?}", cb.to_callback_type()),
                    ))
                } else {
                    Ok(true)
                }
            } else {
                warn!(%uid, "callback has unknow option uid");
                Ok(true)
            }
        })?;
        let has_cmd = set.opt_iter().any(|v| v.match_style(Style::Cmd));

        const MAX_INDEX: u64 = u64::MAX;

        if has_cmd {
            for opt in set.opt_iter() {
                if opt.as_ref().match_style(Style::Pos) {
                    if let Some(index) = opt.as_ref().get_index() {
                        let index = index.calc_index(MAX_INDEX, 1).unwrap_or(MAX_INDEX);
                        if index == 1 && !opt.as_ref().get_optional() {
                            // if we have cmd, can not have force required POS @1
                            return Err(Error::opt_can_not_insert_pos());
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    /// Check if the option is valid.
    fn opt_check(&self, set: &S) -> Result<bool> {
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

    /// Check if the POS is valid.
    /// For which POS is have certainty position, POS has same position are replaceble even it is force reuqired.
    /// For which POS is have uncertainty position, it must be set if it is force reuqired.
    fn pos_check(&self, set: &S) -> Result<bool> {
        // for POS has certainty position, POS has same position are replaceble even it is force reuqired.
        let mut index_map: HashMap<u64, Vec<Uid>> = HashMap::new();
        // for POS has uncertainty position, it must be set if it is force reuqired
        let mut float_vec: Vec<Uid> = vec![];

        for opt in set.opt_iter() {
            if opt.as_ref().match_style(Style::Pos) {
                if let Some(index) = opt.get_index() {
                    match index {
                        OptIndex::Forward(_) | OptIndex::Backward(_) => {
                            if let Some(index) = index.calc_index(u64::MAX, 1) {
                                let entry = index_map.entry(index).or_insert(vec![]);
                                entry.push(opt.get_uid());
                            }
                        }
                        OptIndex::List(v) => {
                            for index in v {
                                let entry = index_map.entry(*index).or_insert(vec![]);
                                entry.push(opt.get_uid());
                            }
                        }
                        OptIndex::Except(_)
                        | OptIndex::Greater(_)
                        | OptIndex::Less(_)
                        | OptIndex::AnyWhere => {
                            float_vec.push(opt.get_uid());
                        }
                        OptIndex::Null => {}
                    }
                }
            }
        }
        let mut names = vec![];

        trace!(?index_map, ?float_vec, "pos check information");
        for (index, uids) in index_map.iter() {
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
            debug!(%pos_valid, "in default pos check @ pos {}", index);
            if !pos_valid {
                return Err(Error::sp_pos_force_require(names.join(" | ")));
            }
            names.clear();
        }
        if !float_vec.is_empty() {
            float_vec
                .iter()
                .filter(|&uid| set.get_opt(*uid).unwrap().check().is_err())
                .for_each(|&uid| {
                    names.push(set.get_opt(uid).unwrap().get_hint().to_owned());
                });
            if !names.is_empty() {
                debug!(?names, "in default float pos check @ pos");
                return Err(Error::sp_pos_force_require(names.join(" | ")));
            }
        }
        Ok(true)
    }

    fn cmd_check(&self, set: &S) -> Result<bool> {
        let mut names = vec![];
        let mut valid = false;

        for opt in set.opt_iter() {
            if opt.as_ref().match_style(Style::Cmd) {
                valid = valid || opt.check().is_ok();
                if valid {
                    break;
                } else {
                    names.push(opt.get_hint().to_owned());
                }
            }
        }
        if !valid && !names.is_empty() {
            return Err(Error::sp_cmd_force_require(names.join(" | ")));
        }
        Ok(true)
    }

    fn post_check(&self, set: &S) -> Result<bool> {
        for opt in set.opt_iter() {
            if opt.as_ref().match_style(Style::Main) {
                opt.check()?;
            }
        }
        Ok(true)
    }

    fn invoke(
        &mut self,
        uid: Uid,
        set: &mut S,
        noa_idx: usize,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        if let Some(callback) = self.callback_store.get_callback_mut(&uid) {
            debug!("calling callback of option<{}>", uid);
            match callback {
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

    fn get_callback(&self) -> &CallbackStore<S> {
        &self.callback_store
    }

    fn get_subscriber_info<I: 'static + Info>(&self) -> &Vec<Box<dyn Info>> {
        &self.subscriber_info
    }

    fn get_noa(&self) -> &Vec<Ustr> {
        &self.noa
    }

    fn get_callback_mut(&mut self) -> &mut CallbackStore<S> {
        &mut self.callback_store
    }

    fn get_subscriber_info_mut(&mut self) -> &mut Vec<Box<dyn Info>> {
        &mut self.subscriber_info
    }

    fn get_noa_mut(&mut self) -> &mut Vec<Ustr> {
        &mut self.noa
    }

    fn reset(&mut self) {
        self.subscriber_info.clear();
        self.callback_store.clear();
        self.noa.clear();
    }
}

impl<S: Set, M: Matcher> Proc<S, M> for SimpleService<S> {
    fn process(&mut self, msg: &mut M, set: &mut S, invoke: bool) -> Result<Vec<ValueKeeper>> {
        match msg.get_style() {
            Style::Boolean | Style::Argument | Style::Multiple => {
                self.matching_opt(msg, set, invoke)
            }
            Style::Pos | Style::Cmd | Style::Main => self.matching_nonopt(msg, set, invoke),
            Style::Other | Style::Null => Ok(vec![]),
        }
    }
}
