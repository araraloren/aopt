pub mod seq;
pub mod single;

use std::fmt::Debug;
use std::slice::Iter;

use crate::ctx::Context;
use crate::err::Result;
use crate::opt::Opt;
use crate::set::Set;
use crate::uid::Uid;

pub use seq::SequenceProc;
pub use single::SingleProc;

pub trait Message: Debug {
    fn msg_uid(&self) -> Uid;
}

pub trait Info: Debug {
    fn uid(&self) -> Uid;
}

#[async_trait::async_trait(?Send)]
pub trait Publisher<M: Message, S: Set> {
    #[cfg(not(feature = "async"))]
    fn publish(&mut self, msg: &mut M, set: &mut S) -> Result<bool>;

    #[cfg(feature = "async")]
    async fn publish(&mut self, msg: &mut M, set: &mut S) -> Result<bool>;

    fn subscriber_iter(&self) -> Iter<'_, Box<dyn Info>>;

    fn reg_subscriber(&mut self, info: Box<dyn Info>);

    fn clr_subscriber(&mut self);
}

pub trait Subscriber<M: Message, S: Set> {
    fn subscribe_from(&self, publisher: &mut dyn Publisher<M, S>);
}

#[async_trait::async_trait(?Send)]
pub trait Proc: Debug {
    fn uid(&self) -> Uid;

    fn add_ctx(&mut self, ctx: Box<dyn Context>);

    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>>;

    #[cfg(not(feature = "async"))]
    fn process(&mut self, opt: &mut dyn Opt) -> Result<Option<usize>>;

    #[cfg(feature = "async")]
    async fn process(&mut self, opt: &mut dyn Opt) -> Result<Option<usize>>;

    fn is_matched(&self) -> bool;

    fn is_comsume_argument(&self) -> bool;

    fn len(&self) -> usize;
}

impl<T: Proc> Message for T {
    fn msg_uid(&self) -> Uid {
        self.uid()
    }
}
