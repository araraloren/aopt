pub(crate) mod context;
#[cfg_attr(feature = "sync", path = "../sync/ctx/invoke.rs")]
#[cfg_attr(not(feature = "sync"), path = "invoke.rs")]
pub(crate) mod invoke;
pub(crate) mod store;

pub use self::context::Ctx;
pub use self::context::InnerCtx;
pub use self::invoke::HandlerCollection;
pub use self::invoke::HandlerEntry;
pub use self::invoke::HandlerEntryThen;
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
pub use __wrapper::wrap_handler_fallback_action;

#[cfg(feature = "sync")]
mod __wrapper {
    use super::*;
    use crate::map::ErasedTy;

    /// Wrap the handler and call the default action of option if return value is `Some()`,
    /// otherwise call the [`fallback`](crate::ctx::Invoker::fallback).
    pub fn wrap_handler_fallback_action<'a, S, H, O, E>(
        mut handler: H,
    ) -> impl FnMut(&mut S, &mut Ctx) -> Result<bool, Error> + 'a
    where
        O: ErasedTy,
        E: Into<Error>,
        S: crate::set::Set,
        SetOpt<S>: Opt,
        H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, E> + Send + Sync + 'a,
    {
        move |set: &mut S, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;

            if val.is_some() {
                let arg = ctx.arg()?.map(|v| v.as_ref());
                let uid = ctx.uid()?;
                let mut act = *set.opt(uid)?.action();

                act.process(uid, set, arg, val)
            } else {
                Invoker::fallback(set, ctx)
            }
        }
    }

    /// Wrap the handler and call the [`process`](Store::process) of `store` if return value is `Some()`,
    /// otherwise call the [`fallback`](crate::ctx::Invoker::fallback).
    pub fn wrap_handler_fallback<'a, Set, H, O, E, S>(
        mut handler: H,
        mut store: S,
    ) -> impl FnMut(&mut Set, &mut Ctx) -> Result<bool, Error> + 'a
    where
        O: ErasedTy,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        E: Into<Error>,
        S: Store<Set, O, Ret = bool, Error = E> + Send + Sync + 'a,
        H: FnMut(&mut Set, &mut Ctx) -> Result<Option<O>, E> + Send + Sync + 'a,
    {
        move |set: &mut Set, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;

            if val.is_some() {
                let arg = ctx.arg()?.map(|v| v.as_ref());
                let uid = ctx.uid()?;

                store.process(uid, set, arg, val).map_err(Into::into)
            } else {
                Invoker::fallback(set, ctx)
            }
        }
    }

    /// Wrap the handler and call the default action of option.
    pub fn wrap_handler_action<'a, S, H, O, E>(
        mut handler: H,
    ) -> impl FnMut(&mut S, &mut Ctx) -> Result<bool, Error> + 'a
    where
        O: ErasedTy,
        E: Into<Error>,
        S: crate::set::Set,
        SetOpt<S>: Opt,
        H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, E> + Send + Sync + 'a,
    {
        move |set: &mut S, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;
            let arg = ctx.arg()?.map(|v| v.as_ref());
            let uid = ctx.uid()?;
            let mut act = *set.opt(uid)?.action();

            act.process(uid, set, arg, val)
        }
    }

    /// Wrap the handler and call the [`process`](Store::process) of given `store` on return value of `handler`.
    pub fn wrap_handler<'a, Set, H, O, E, S>(
        mut handler: H,
        mut store: S,
    ) -> impl FnMut(&mut Set, &mut Ctx) -> Result<bool, Error> + 'a
    where
        E: Into<Error>,
        S: Store<Set, O, Ret = bool, Error = E> + Send + Sync + 'a,
        H: FnMut(&mut Set, &mut Ctx) -> Result<Option<O>, E> + Send + Sync + 'a,
    {
        Box::new(move |set: &mut Set, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;
            let arg = ctx.arg()?.map(|v| v.as_ref());
            let uid = ctx.uid()?;

            store.process(uid, set, arg, val).map_err(Into::into)
        })
    }
}

#[cfg(not(feature = "sync"))]
mod __wrapper {
    use super::*;
    use crate::map::ErasedTy;

    /// Wrap the handler and call the default action of option if return value is `Some()`,
    /// otherwise call the [`fallback`](crate::ctx::Invoker::fallback).
    pub fn wrap_handler_fallback_action<'a, Set, H, O, E>(
        mut handler: H,
    ) -> impl FnMut(&mut Set, &mut Ctx) -> Result<bool, Error> + 'a
    where
        O: ErasedTy,
        E: Into<Error>,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        H: FnMut(&mut Set, &mut Ctx) -> Result<Option<O>, E> + 'a,
    {
        move |set: &mut Set, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;

            if val.is_some() {
                let arg = ctx.arg()?.map(|v| v.as_ref());
                let uid = ctx.uid()?;
                let mut act = *set.opt(uid)?.action();

                act.process(uid, set, arg, val)
            } else {
                Invoker::fallback(set, ctx)
            }
        }
    }

    /// Wrap the handler and call the [`process`](Store::process) of `store` if return value is `Some()`,
    /// otherwise call the [`fallback`](crate::ctx::Invoker::fallback).
    pub fn wrap_handler_fallback<'a, Set, H, O, E, S>(
        mut handler: H,
        mut store: S,
    ) -> impl FnMut(&mut Set, &mut Ctx) -> Result<bool, Error> + 'a
    where
        O: ErasedTy,
        E: Into<Error>,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        S: Store<Set, O, Ret = bool, Error = E> + 'a,
        H: FnMut(&mut Set, &mut Ctx) -> Result<Option<O>, E> + 'a,
    {
        move |set: &mut Set, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;

            if val.is_some() {
                let arg = ctx.arg()?.map(|v| v.as_ref());
                let uid = ctx.uid()?;

                store.process(uid, set, arg, val).map_err(Into::into)
            } else {
                Invoker::fallback(set, ctx)
            }
        }
    }

    /// Wrap the handler and call the default action of option.
    pub fn wrap_handler_action<'a, Set, H, O, E>(
        mut handler: H,
    ) -> impl FnMut(&mut Set, &mut Ctx) -> Result<bool, Error> + 'a
    where
        O: ErasedTy,
        E: Into<Error>,
        Set: crate::set::Set,
        SetOpt<Set>: Opt,
        H: FnMut(&mut Set, &mut Ctx) -> Result<Option<O>, E> + 'a,
    {
        move |set: &mut Set, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;
            let arg = ctx.arg()?.map(|v| v.as_ref());
            let uid = ctx.uid()?;
            let mut act = *set.opt(uid)?.action();

            act.process(uid, set, arg, val)
        }
    }

    /// Wrap the handler and call the [`process`](Store::process) of given `store` on return value of `handler`.
    pub fn wrap_handler<'a, Set, H, O, E, S>(
        mut handler: H,
        mut store: S,
    ) -> impl FnMut(&mut Set, &mut Ctx) -> Result<bool, Error> + 'a
    where
        O: ErasedTy,
        E: Into<Error>,
        S: Store<Set, O, Ret = bool, Error = E> + 'a,
        H: FnMut(&mut Set, &mut Ctx) -> Result<Option<O>, E> + 'a,
    {
        Box::new(move |set: &mut Set, ctx: &mut Ctx| {
            let val = (handler)(set, ctx).map_err(Into::into)?;
            let arg = ctx.arg()?.map(|v| v.as_ref());
            let uid = ctx.uid()?;

            store.process(uid, set, arg, val).map_err(Into::into)
        })
    }
}
