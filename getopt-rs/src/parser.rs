use std::fmt::Debug;
use std::slice::Iter;

use crate::err::Result;
use crate::opt::{OptCallback, OptValue};
use crate::proc::{Info, Message, Proc, Publisher};
use crate::set::Set;
use crate::uid::Uid;

#[async_trait::async_trait(?Send)]
pub trait Parser<S>: Debug
where
    S: Set,
{
    #[cfg(not(feature = "async"))]
    fn parse(&mut self, set: S, iter: impl Iterator<Item = String>) -> Result<Option<bool>>;

    #[cfg(feature = "async")]
    async fn parse(&mut self, set: S, iter: impl Iterator<Item = String>) -> Result<Option<bool>>;

    fn add_callback(&mut self, uid: Uid, callback: OptCallback);

    #[cfg(not(feature = "async"))]
    fn invoke_callback(&self, uid: Uid, set: &mut S, noa_index: usize) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn invoke_callback(
        &self,
        uid: Uid,
        set: &mut S,
        noa_index: usize,
    ) -> Result<Option<OptValue>>;

    fn pre_check(&self) -> Result<bool>;

    fn check_opt(&self) -> Result<bool>;

    fn check_nonopt(&self) -> Result<bool>;

    fn post_check(&self) -> Result<bool>;

    fn subscriber_iter(&self) -> Iter<'_, Box<dyn Info>>;

    fn reg_subscriber(&mut self, info: Box<dyn Info>);

    fn clr_subscriber(&mut self);

    fn reset(&mut self);
}

impl<S, P, T: Parser<S>> Publisher<P, S> for T
where
    S: Set,
    P: Proc,
{
    #[cfg(not(feature = "async"))]
    fn publish(&mut self, msg: &mut P, set: &mut S) -> Result<bool> {
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

    fn subscriber_iter(&self) -> Iter<'_, Box<dyn Info>> {
        T::subscriber_iter(&self)
    }
}
