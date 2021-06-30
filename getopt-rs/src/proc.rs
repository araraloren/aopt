use async_trait::async_trait;
use std::fmt::Debug;

use crate::ctx::Context;
use crate::err::{Error, Result};
use crate::opt::Opt;
use crate::uid::Uid;

pub trait Message: Debug {
    fn msg_uid(&self) -> Uid;
}

pub trait Info: Debug {
    fn uid(&self) -> Uid;
}

#[async_trait(?Send)]
pub trait Publisher<M: Message> {
    #[cfg(not(feature = "async"))]
    fn publish(&mut self, msg: M) -> Result<bool>;

    #[cfg(feature = "async")]
    async fn publish(&mut self, msg: M) -> Result<bool>;

    fn reg_subscriber(&mut self, info: Box<dyn Info>);

    fn clean(&mut self);
}

pub trait Subscriber<M: Message> {
    fn subscribe_from(&self, publisher: &mut dyn Publisher<M>);
}

#[async_trait(?Send)]
pub trait Proc: Debug {
    fn uid(&self) -> Uid;

    fn app_ctx(&mut self, ctx: Box<dyn Context>);

    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>>;

    #[cfg(not(feature = "async"))]
    fn process(&mut self, opt: &mut dyn Opt) -> Result<Option<u64>>;

    #[cfg(feature = "async")]
    async fn process(&mut self, opt: &mut dyn Opt) -> Result<Option<u64>>;

    fn is_matched(&self) -> bool;

    fn len(&self) -> usize;
}

impl<T: Proc> Message for T {
    fn msg_uid(&self) -> Uid {
        self.uid()
    }
}
