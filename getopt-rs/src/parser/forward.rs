use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;

use super::GenStyle;
use super::{HashMapIter, SliceIter};
use super::{Parser, ReturnValue};
use crate::arg::ArgStream;
use crate::err::Result;
use crate::opt::{OptCallback, OptValue};
use crate::proc::{Info, NonOptCtxProc, OptCtxProc, Proc, Subscriber};
use crate::set::Set;
use crate::uid::{Generator, Uid};

#[derive(Debug)]
pub struct ForwardParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    uid_gen: G,

    subscriber_info: Vec<Box<dyn Info>>,

    callback: HashMap<Uid, RefCell<OptCallback>>,

    noa: Vec<String>,

    gen_style_order: Vec<GenStyle>,

    marker: PhantomData<S>,
}

impl<S, G> Default for ForwardParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    fn default() -> Self {
        Self {
            uid_gen: G::default(),
            subscriber_info: vec![],
            callback: HashMap::new(),
            noa: vec![],
            gen_style_order: vec![
                GenStyle::GSEqualWithValue,
                GenStyle::GSArgument,
                GenStyle::GSBoolean,
                GenStyle::GSMultipleOption,
                GenStyle::GSEmbeddedValue,
            ],
            marker: PhantomData::default(),
        }
    }
}

impl<S, G> ForwardParser<S, G>
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

#[async_trait::async_trait(?Send)]
impl<S, G> Parser<S> for ForwardParser<S, G>
where
    S: Set + Default,
    G: Generator + Debug + Default,
{
    #[cfg(not(feature = "async"))]
    fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>> {
        use crate::proc::Publisher;

        let mut argstream = ArgStream::from(iter);
        let mut set = set;
        let mut iter = argstream.iter_mut();

        // copy the prefix, so we don't need borrow set
        let prefix: Vec<String> = set.get_prefix().iter().map(|v| v.clone()).collect();

        set.subscribe_from(self);
        self.pre_check(&set)?;

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
                    for gen_style in self.gen_style_order.clone() {
                        if let Some(ret) = gen_style.gen_opt::<OptCtxProc>(arg) {
                            let mut proc: Box<dyn Proc> = Box::new(ret);

                            if self.publish(&mut proc, &mut set)? {
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
                debug!("!!! Not matching {:?}", &arg);
                if let Some(noa) = &arg.current {
                    self.noa.push(noa.clone());
                }
            }
        }

        self.check_opt(&set)?;

        let noa_count = self.noa.len();

        if noa_count > 0 {
            let gen_style = GenStyle::GSNonCmd;

            debug!("Start process {:?} ...", &gen_style);
            if let Some(ret) =
                gen_style.gen_nonopt::<NonOptCtxProc>(&self.noa[0], noa_count as u64, 1)
            {
                let mut proc: Box<dyn Proc> = Box::new(ret);

                self.publish(&mut proc, &mut set)?;
            }

            let gen_style = GenStyle::GSNonPos;

            debug!("Start process {:?} ...", &gen_style);
            for index in 1..=noa_count {
                if let Some(ret) = gen_style.gen_nonopt::<NonOptCtxProc>(
                    &self.noa[index - 1],
                    noa_count as u64,
                    index as u64,
                ) {
                    let mut proc: Box<dyn Proc> = Box::new(ret);

                    self.publish(&mut proc, &mut set)?;
                }
            }
        }

        self.check_nonopt(&set)?;

        let gen_style = GenStyle::GSNonMain;

        debug!("Start process {:?} ...", &gen_style);
        if let Some(ret) =
            gen_style.gen_nonopt::<NonOptCtxProc>(&String::new(), noa_count as u64, 1)
        {
            let mut proc: Box<dyn Proc> = Box::new(ret);

            self.publish(&mut proc, &mut set)?;
        }

        self.post_check(&set)?;

        Ok(Some(ReturnValue {
            set: set,
            noa: &self.noa,
        }))
    }

    #[cfg(feature = "async")]
    async fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>> {
        use crate::proc::Publisher;

        let mut argstream = ArgStream::from(iter);
        let mut set = set;
        let mut iter = argstream.iter_mut();

        // copy the prefix, so we don't need borrow set
        let prefix: Vec<String> = set.get_prefix().iter().map(|v| v.clone()).collect();

        set.subscribe_from(self);
        self.pre_check(&set)?;

        // iterate the Arguments, generate option context
        // send it to Publisher
        debug!("Start process option ...");
        while let Some(arg) = iter.next() {
            let mut matched = false;

            debug!("Get next Argument => {:?}", &arg);
            if let Ok(ret) = arg.parse(&prefix) {
                if ret {
                    debug!(" ... parsed: {:?}", &arg);
                    for gen_style in self.gen_style_order.clone() {
                        if let Some(ret) = gen_style.gen_opt::<OptCtxProc>(arg) {
                            let mut proc: Box<dyn Proc> = Box::new(ret);

                            if let Ok(_) = self.publish(&mut proc, &mut set) {
                                if proc.is_matched() && proc.is_comsume_argument() {
                                    matched = true;
                                }
                            }
                        }
                    }
                }
            }
            if matched {
                iter.next();
            } else {
                if let Some(noa) = &arg.current {
                    self.noa.push(noa.clone());
                }
            }
        }

        self.check_opt(&set)?;

        let noa_count = self.noa.len();

        if noa_count > 0 {
            let gen_style = GenStyle::GSNonCmd;

            debug!("Start process {:?} ...", &gen_style);
            if let Some(ret) =
                gen_style.gen_nonopt::<NonOptCtxProc>(&self.noa[0], noa_count as u64, 1)
            {
                let mut proc: Box<dyn Proc> = Box::new(ret);

                if let Ok(ret) = self.publish(&mut proc, &mut set) {
                    debug!("ret = {:?}", ret);
                }
            }

            let gen_style = GenStyle::GSNonPos;

            debug!("Start process {:?} ...", &gen_style);
            for index in 1..=noa_count {
                if let Some(ret) = gen_style.gen_nonopt::<NonOptCtxProc>(
                    &self.noa[index - 1],
                    noa_count as u64,
                    index as u64,
                ) {
                    let mut proc: Box<dyn Proc> = Box::new(ret);

                    if let Ok(ret) = self.publish(&mut proc, &mut set) {
                        debug!("ret = {:?}", ret);
                    }
                }
            }
        }

        self.check_nonopt(&set)?;

        let gen_style = GenStyle::GSNonMain;

        debug!("Start process {:?} ...", &gen_style);
        if let Some(ret) =
            gen_style.gen_nonopt::<NonOptCtxProc>(&String::new(), noa_count as u64, 1)
        {
            let mut proc: Box<dyn Proc> = Box::new(ret);

            if let Ok(ret) = self.publish(&mut proc, &mut set) {
                debug!("ret = {:?}", ret);
            }
        }

        self.post_check(&set)?;
        todo!();
        Ok(None)
    }

    #[cfg(not(feature = "async"))]
    fn invoke_callback(&self, uid: Uid, set: &mut S, noa_index: usize) -> Result<Option<OptValue>> {
        if let Some(callback) = self.callback.get(&uid) {
            debug!("calling callback of option<{}>", uid);
            match callback.borrow_mut().deref_mut() {
                OptCallback::Opt(cb) => cb.as_mut().call(uid, set),
                OptCallback::OptMut(cb) => cb.as_mut().call(uid, set),
                OptCallback::Pos(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_index - 1], noa_index as u64)
                }
                OptCallback::PosMut(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_index - 1], noa_index as u64)
                }
                OptCallback::Main(cb) => cb.as_mut().call(uid, set, &self.noa),
                OptCallback::MainMut(cb) => cb.as_mut().call(uid, set, &self.noa),
                OptCallback::Null => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "async")]
    async fn invoke_callback(
        &self,
        uid: Uid,
        set: &mut S,
        noa_index: usize,
    ) -> Result<Option<OptValue>> {
        if let Some(callback) = self.callback.get(&uid) {
            debug!("calling callback of option<{}>", uid);
            match callback.borrow_mut().deref_mut() {
                OptCallback::Opt(cb) => cb.as_mut().call(uid, set).await,
                OptCallback::OptMut(cb) => cb.as_mut().call(uid, set).await,
                OptCallback::Pos(cb) => cb.as_mut().call(uid, set, &self.noa[noa_index]).await,
                OptCallback::PosMut(cb) => cb.as_mut().call(uid, set, &self.noa[noa_index]).await,
                OptCallback::Main(cb) => cb.as_mut().call(uid, set, &self.noa).await,
                OptCallback::MainMut(cb) => cb.as_mut().call(uid, set, &self.noa).await,
                OptCallback::Null => Ok(None),
            }
        } else {
            Ok(None)
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

    fn subscriber_iter(&self) -> SliceIter<'_, Box<dyn Info>> {
        self.subscriber_info.iter()
    }

    fn reg_subscriber(&mut self, info: Box<dyn Info>) {
        self.subscriber_info.push(info);
    }

    fn clr_subscriber(&mut self) {
        self.subscriber_info.clear();
    }

    fn reset(&mut self) {
        self.uid_gen.reset();
        self.noa.clear();
        // don't know why this not working
        // self.clr_subscriber();
        self.subscriber_info.clear();
    }
}
