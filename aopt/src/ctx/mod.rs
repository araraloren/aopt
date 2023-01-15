pub(crate) mod context;
pub(crate) mod extract;
pub(crate) mod handler;
#[cfg_attr(feature = "sync", path = "../sync/ctx/invoke.rs")]
#[cfg_attr(not(feature = "sync"), path = "invoke.rs")]
pub(crate) mod invoke;
pub(crate) mod store;

pub use self::context::Ctx;
pub use self::context::InnerCtx;
pub use self::extract::Extract;
pub use self::handler::Handler;
pub use self::invoke::HandlerEntry;
pub use self::invoke::InvokeHandler;
pub use self::invoke::Invoker;
pub use self::store::NullStore;
pub use self::store::Store;
pub use self::store::VecStore;

use crate::opt::Opt;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::Error;

pub use __wrapper::wrap_handler;
pub use __wrapper::wrap_handler_action;
pub use __wrapper::wrap_handler_fallback;

#[cfg(feature = "sync")]
mod __wrapper {

    use super::*;

    /// Wrap the handler and call the default action of option if return value is `Some()`,
    /// otherwise call the [`fallback`](crate::ctx::Invoker::fallback).
    pub fn wrap_handler_fallback<Set, Ser, A, O, H, E>(
        mut handler: H,
    ) -> impl FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error>
    where
        E: Into<Error>,
        O: Send + Sync + 'static,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        A: Extract<Set, Ser, Error = E> + Send + Sync,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = E> + Send + Sync + 'static,
    {
        move |set: &mut Set, ser: &mut Ser, ctx: &Ctx| {
            let val = handler
                .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                .map_err(Into::into)?;

            if val.is_some() {
                let arg = ctx.arg()?;
                let arg = arg.as_ref().map(|v| v.as_ref());
                let uid = ctx.uid()?;
                let mut act = *set.opt(uid)?.action();

                act.process(uid, set, ser, arg, val)
            } else {
                Invoker::fallback(set, ser, ctx)
            }
        }
    }

    /// Wrap the handler and call the default action of option.
    pub fn wrap_handler_action<Set, Ser, A, O, H, E>(
        mut handler: H,
    ) -> impl FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error>
    where
        E: Into<Error>,
        O: Send + Sync + 'static,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        A: Extract<Set, Ser, Error = E> + Send + Sync,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = E> + Send + Sync + 'static,
    {
        move |set: &mut Set, ser: &mut Ser, ctx: &Ctx| {
            let val = handler
                .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                .map_err(Into::into)?;
            let arg = ctx.arg()?;
            let arg = arg.as_ref().map(|v| v.as_ref());
            let uid = ctx.uid()?;
            let mut act = *set.opt(uid)?.action();

            act.process(uid, set, ser, arg, val)
        }
    }

    /// Wrap the handler and call the [`process`](Store::process) of given `store` on return value of `handler`.
    pub fn wrap_handler<Set, Ser, A, O, H, T, E>(
        mut handler: H,
        mut store: T,
    ) -> impl FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error>
    where
        E: Into<Error>,
        A: Extract<Set, Ser, Error = E> + Send + Sync,
        T: Store<Set, Ser, O, Ret = bool, Error = E> + Send + Sync + 'static,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = E> + Send + Sync + 'static,
    {
        Box::new(move |set: &mut Set, ser: &mut Ser, ctx: &Ctx| {
            let ext_args = A::extract(set, ser, ctx).map_err(Into::into)?;
            let val = handler.invoke(set, ser, ext_args).map_err(Into::into)?;
            let arg = ctx.arg()?;
            let arg = arg.as_ref().map(|v| v.as_ref());
            let uid = ctx.uid()?;

            store.process(uid, set, ser, arg, val).map_err(Into::into)
        })
    }
}

#[cfg(not(feature = "sync"))]
mod __wrapper {
    use super::*;

    /// Wrap the handler and call the default action of option if return value is `Some()`,
    /// otherwise call the [`fallback`](crate::ctx::Invoker::fallback).
    pub fn wrap_handler_fallback<Set, Ser, A, O, H, E>(
        mut handler: H,
    ) -> impl FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error>
    where
        O: 'static,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        E: Into<Error>,
        A: Extract<Set, Ser, Error = E>,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = E> + 'static,
    {
        move |set: &mut Set, ser: &mut Ser, ctx: &Ctx| {
            let val = handler
                .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                .map_err(Into::into)?;

            if val.is_some() {
                let arg = ctx.arg()?;
                let arg = arg.as_ref().map(|v| v.as_ref());
                let uid = ctx.uid()?;
                let mut act = *set.opt(uid)?.action();

                act.process(uid, set, ser, arg, val)
            } else {
                Invoker::fallback(set, ser, ctx)
            }
        }
    }

    /// Wrap the handler and call the default action of option.
    pub fn wrap_handler_action<Set, Ser, A, O, H, E>(
        mut handler: H,
    ) -> impl FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error>
    where
        O: 'static,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        E: Into<Error>,
        A: Extract<Set, Ser, Error = E>,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = E> + 'static,
    {
        move |set: &mut Set, ser: &mut Ser, ctx: &Ctx| {
            let val = handler
                .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                .map_err(Into::into)?;
            let arg = ctx.arg()?;
            let arg = arg.as_ref().map(|v| v.as_ref());
            let uid = ctx.uid()?;
            let mut act = *set.opt(uid)?.action();

            act.process(uid, set, ser, arg, val)
        }
    }

    /// Wrap the handler and call the [`process`](Store::process) of given `store` on return value of `handler`.
    pub fn wrap_handler<Set, Ser, A, O, H, T, E>(
        mut handler: H,
        mut store: T,
    ) -> impl FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error>
    where
        E: Into<Error>,
        A: Extract<Set, Ser, Error = E>,
        T: Store<Set, Ser, O, Ret = bool, Error = E> + 'static,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = E> + 'static,
    {
        Box::new(move |set: &mut Set, ser: &mut Ser, ctx: &Ctx| {
            let ext_args = A::extract(set, ser, ctx).map_err(Into::into)?;
            let val = handler.invoke(set, ser, ext_args).map_err(Into::into)?;
            let arg = ctx.arg()?;
            let arg = arg.as_ref().map(|v| v.as_ref());
            let uid = ctx.uid()?;

            store.process(uid, set, ser, arg, val).map_err(Into::into)
        })
    }
}
