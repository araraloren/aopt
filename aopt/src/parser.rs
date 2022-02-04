mod delay_policy;
mod forward_policy;
mod pre_policy;
mod service;
mod state;
// pub(crate) mod testutil;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use ustr::Ustr;

use crate::arg::Argument;
use crate::err::Result;
use crate::opt::{OptCallback, OptValue};
use crate::proc::{Info, Matcher};
use crate::set::Set;
use crate::uid::Uid;

pub use delay_policy::DelayPolicy;
pub use forward_policy::ForwardPolicy;
pub use pre_policy::PrePolicy;
pub use service::DefaultService;
pub use state::ParserState;

#[derive(Debug, Clone)]
pub struct ValueKeeper {
    pub id: Uid,
    pub index: usize,
    pub value: OptValue,
}

pub trait Policy<S: Set, SS: Service> {
    fn parse(
        &mut self,
        set: &mut S,
        service: &mut SS,
        iter: &mut dyn Iterator<Item = Argument>,
    ) -> Result<bool>;
}

pub trait Service {
    fn gen_opt<M: Matcher>(&self, arg: &Argument, style: &ParserState) -> Result<Option<M>>;

    fn gen_nonopt<M: Matcher>(
        &self,
        noa: &Ustr,
        total: usize,
        current: usize,
        style: &ParserState,
    ) -> Result<Option<M>>;

    fn matching<M: Matcher, S: Set>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        invoke: bool,
    ) -> Result<Vec<ValueKeeper>>;

    fn pre_check<S: Set>(&self, set: &S) -> Result<bool>;

    fn opt_check<S: Set>(&self, set: &S) -> Result<bool>;

    fn nonopt_check<S: Set>(&self, set: &S) -> Result<bool>;

    fn post_check<S: Set>(&self, set: &S) -> Result<bool>;

    fn invoke<S: Set>(
        &self,
        uid: Uid,
        set: &mut S,
        noa_idx: usize,
        optvalue: OptValue,
    ) -> Result<Option<OptValue>>;

    fn get_callback(&self) -> &HashMap<Uid, RefCell<OptCallback>>;

    fn get_subscriber_info<I: 'static + Info>(&self) -> &Vec<Box<dyn Info>>;

    fn get_noa(&self) -> &Vec<Ustr>;

    fn get_callback_mut(&mut self) -> &mut HashMap<Uid, RefCell<OptCallback>>;

    fn get_subscriber_info_mut(&mut self) -> &mut Vec<Box<dyn Info>>;

    fn get_noa_mut(&mut self) -> &mut Vec<Ustr>;

    fn reset(&mut self);
}

#[derive(Debug)]
pub struct Parser<S: Set, SS: Service, P: Policy<S, SS>> {
    policy: P,
    service: SS,
    set: S,
}

impl<S: Set, SS: Service, P: Policy<S, SS>> Parser<S, SS, P> {
    pub fn parse(&mut self, iter: &mut dyn Iterator<Item = Argument>) -> Result<bool> {
        let service = &mut self.service;
        let policy = &mut self.policy;
        let set = &mut self.set;

        policy.parse(set, service, iter)
    }
}
