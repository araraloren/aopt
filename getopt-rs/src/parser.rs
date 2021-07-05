pub mod check;
pub mod forward;
pub mod gen_style;

use std::cell::RefCell;
use std::fmt::Debug;

pub(crate) use std::collections::hash_map::Iter as HashMapIter;
pub(crate) use std::slice::Iter as SliceIter;

use crate::err::Result;
use crate::opt::{OptCallback, OptValue};
use crate::proc::{Info, Message, Proc, Publisher};
use crate::set::Set;
use crate::uid::Uid;

pub use gen_style::GenStyle;

#[async_trait::async_trait(?Send)]
pub trait Parser<S>: Debug
where
    S: Set,
    Self: Sized,
{
    #[cfg(not(feature = "async"))]
    fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>>;

    #[cfg(feature = "async")]
    async fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>>;

    fn add_callback(&mut self, uid: Uid, callback: OptCallback);

    fn get_callback(&self, uid: Uid) -> Option<&RefCell<OptCallback>>;

    fn callback_iter(&self) -> HashMapIter<'_, Uid, RefCell<OptCallback>>;

    #[cfg(not(feature = "async"))]
    fn invoke_callback(&self, uid: Uid, set: &mut S, noa_index: usize) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn invoke_callback(
        &mut self,
        uid: Uid,
        set: &mut S,
        noa_index: usize,
    ) -> Result<Option<OptValue>>;

    fn pre_check(&self, set: &S) -> Result<bool> {
        check::default_pre_check(set, self)
    }

    fn check_opt(&self, set: &S) -> Result<bool> {
        check::default_opt_check(set, self)
    }

    fn check_nonopt(&self, set: &S) -> Result<bool> {
        check::default_nonopt_check(set, self)
    }

    fn post_check(&self, set: &S) -> Result<bool> {
        check::default_post_check(set, self)
    }

    fn subscriber_iter(&self) -> SliceIter<'_, Box<dyn Info>>;

    fn reg_subscriber(&mut self, info: Box<dyn Info>);

    fn clr_subscriber(&mut self);

    fn reset(&mut self);
}

impl<S, T: Parser<S>> Publisher<Box<dyn Proc>, S> for T
where
    S: Set,
{
    #[cfg(not(feature = "async"))]
    fn publish(&mut self, msg: &mut Box<dyn Proc>, set: &mut S) -> Result<bool> {
        let proc = msg;

        debug!("Got message<{}>: {:?}", &proc.msg_uid(), &proc);
        for info in self.subscriber_iter() {
            let opt = set.get_opt_mut(info.uid()).unwrap();
            let res = proc.process(opt.as_mut())?;

            if let Some(noa_index) = res {
                let invoke_callback = opt.is_need_invoke();

                if invoke_callback {
                    let ret = self.invoke_callback(info.uid(), set, noa_index)?;
                    let opt = set.get_opt_mut(info.uid()).unwrap();

                    // need try to borrow opt once more, cause the borrow check
                    opt.set_callback_ret(ret)?;
                }
            }
            if proc.is_matched() {
                debug!("Proc<{}> matched", proc.msg_uid());
                break;
            }
        }
        Ok(proc.is_matched())
    }

    #[cfg(feature = "async")]
    async fn publish(&mut self, msg: &mut P, set: &mut S) -> Result<bool> {
        let proc = msg;

        debug!("Got message<{}>: {:?}", &proc.msg_uid(), &proc);
        for info in self.subscriber_iter() {
            let opt = set.get_opt_mut(info.uid()).unwrap();
            let res = proc.process(opt.as_mut()).await?;

            if let Some(noa_index) = res {
                let invoke_callback = opt.is_need_invoke();

                if invoke_callback {
                    let ret = self.invoke_callback(info.uid(), set, noa_index).await?;
                    let opt = set.get_opt_mut(info.uid()).unwrap();

                    // need try to borrow opt once more, cause the borrow check
                    opt.set_callback_ret(ret)?;
                }
            }
            if proc.is_matched() {
                debug!("Proc<{}> matched", proc.msg_uid());
                break;
            }
        }
        Ok(proc.is_matched())
    }

    fn reg_subscriber(&mut self, info: Box<dyn Info>) {
        T::reg_subscriber(self, info);
    }

    fn clr_subscriber(&mut self) {
        T::clr_subscriber(self);
    }

    fn subscriber_iter(&self) -> SliceIter<'_, Box<dyn Info>> {
        T::subscriber_iter(&self)
    }
}

#[derive(Debug, Default)]
pub struct ReturnValue<S: Set> {
    pub noa: Vec<String>,
    pub set: S,
}
