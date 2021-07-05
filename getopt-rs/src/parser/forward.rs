use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use super::GenStyle;
use super::{HashMapIter, SliceIter};
use super::{Parser, ReturnValue};
use crate::arg::ArgStream;
use crate::err::{Error, Result};
use crate::opt::{OptCallback, OptValue};
use crate::proc::{Info, Proc, SequenceProc, SingleProc, Subscriber};
use crate::set::Set;
use crate::uid::{Generator, Uid};

#[derive(Debug)]
pub struct ForwardParser<G>
where
    G: Generator + Debug + Default,
{
    uid_gen: G,

    subscriber_info: Vec<Box<dyn Info>>,

    callback: HashMap<Uid, RefCell<OptCallback>>,

    noa: Vec<String>,

    gen_style_order: Vec<GenStyle>,
}

impl<G> Default for ForwardParser<G>
where
    G: Generator + Debug + Default,
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
        }
    }
}

impl<G> ForwardParser<G>
where
    G: Generator + Debug + Default,
{
    pub fn new(uid_gen: G) -> Self {
        Self {
            uid_gen,
            ..Self::default()
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<S, G> Parser<S> for ForwardParser<G>
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
        let prefix: Vec<String> = set.get_prefix().iter().map(|v| v.clone()).collect();

        set.subscribe_from(self);
        self.pre_check(&set)?;

        while let Some(arg) = argstream.iter_mut().next() {
            let mut proc: Option<Box<dyn Proc>> = None;

            if let Ok(ret) = arg.parse(&prefix) {
                if ret {
                    for gen_style in self.gen_style_order.clone() {
                        if let Some(ret) = gen_style.gen_opt::<SequenceProc>(arg) {
                            proc = Some(Box::new(ret));
                        }
                    }
                }
            }

            if let Some(proc) = proc.as_mut() {
                if let Ok(ret) = self.publish(proc, &mut set) {
                    if ret {}
                }
            }
        }
        Ok(None)
    }

    fn add_callback(&mut self, uid: Uid, callback: OptCallback) {
        self.callback.insert(uid, RefCell::new(callback));
    }

    fn callback_iter(&self) -> HashMapIter<'_, Uid, RefCell<OptCallback>> {
        self.callback.iter()
    }

    fn get_callback(&self, uid: Uid) -> Option<&RefCell<OptCallback>> {
        self.callback.get(&uid)
    }

    #[cfg(not(feature = "async"))]
    fn invoke_callback(&self, uid: Uid, set: &mut S, noa_index: usize) -> Result<Option<OptValue>> {
        if let Some(callback) = self.callback.get(&uid) {
            debug!("calling callback of option<{}>", uid);
            match callback.borrow_mut().deref_mut() {
                OptCallback::Opt(cb) => cb.as_mut().call(uid, set),
                OptCallback::OptMut(cb) => cb.as_mut().call(uid, set),
                OptCallback::Pos(cb) => cb.as_mut().call(uid, set, &self.noa[noa_index]),
                OptCallback::PosMut(cb) => cb.as_mut().call(uid, set, &self.noa[noa_index]),
                OptCallback::Main(cb) => cb.as_mut().call(uid, set, &self.noa),
                OptCallback::MainMut(cb) => cb.as_mut().call(uid, set, &self.noa),
                OptCallback::Null => Ok(None),
            }
        } else {
            Ok(None)
        }
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
    }
}
