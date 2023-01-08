pub(crate) mod context;
pub(crate) mod extract;
pub(crate) mod handler;
#[cfg_attr(feature = "sync", path = "sync/ctx/invoke.rs")]
#[cfg_attr(not(feature = "sync"), path = "invoke.rs")]
pub(crate) mod invoke;
#[cfg_attr(feature = "sync", path = "../sync/ctx/store.rs")]
#[cfg_attr(not(feature = "sync"), path = "store.rs")]
pub(crate) mod store;

pub use self::context::Ctx;
pub use self::extract::Extract;
pub use self::handler::Handler;
pub use self::invoke::HandlerEntry;
pub use self::invoke::InvokeHandler;
pub use self::invoke::Invoker;
pub use self::store::NullStore;
pub use self::store::Store;
pub use self::store::VecStore;

use crate::opt::Opt;
use crate::ser::Services;
use crate::set::Set;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::Error;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        /// Wrap the handler and call the default action of option if return value is `Some()`,
        /// otherwise call the [`fallback`](crate::ser::InvokeService::fallback).
        pub fn wrap_handler_fallback<S, A, O, H, E>(
            mut handler: H,
        ) -> impl FnMut(&mut S, &mut Services, &Ctx) -> Result<Option<()>, Error>
        where
            O: Send + Sync + 'static,
            S: Set,
            SetOpt<S>: Opt,
            E: Into<Error>,
            A: Extract<S, Error = E> + Send + Sync,
            H: Handler<S, A, Output = Option<O>, Error = E> + Send + Sync + 'static,
        {
            move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
                let val = handler
                    .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                    .map_err(Into::into)?;

                if val.is_some() {
                    let arg = ctx.arg();
                    let arg = arg.as_ref().map(|v| v.as_ref());
                    let uid = ctx.uid();
                    let mut act = *set.opt(uid)?.action();

                    act.process(uid, set, ser, arg, val)
                } else {
                    Invoker::fallback(set, ser, ctx)
                }
            }
        }

        /// Wrap the handler and call the default action of option.
        pub fn wrap_handler_action<S, A, O, H, E>(
            mut handler: H,
        ) -> impl FnMut(&mut S, &mut Services, &Ctx) -> Result<Option<()>, Error>
        where
            O: Send + Sync + 'static,
            S: Set,
            SetOpt<S>: Opt,
            E: Into<Error>,
            A: Extract<S, Error = E> + Send + Sync,
            H: Handler<S, A, Output = Option<O>, Error = E> + Send + Sync + 'static,
        {
            move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
                let val = handler
                    .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                    .map_err(Into::into)?;
                let arg = ctx.arg();
                let arg = arg.as_ref().map(|v| v.as_ref());
                let uid = ctx.uid();
                let mut act = *set.opt(uid)?.action();

                act.process(uid, set, ser, arg, val)
            }
        }

        /// Wrap the handler and call the [`process`](Store::process) of given `store` on return value of `handler`.
        pub fn wrap_handler<S, A, O, R, H, T, E>(
            mut handler: H,
            mut store: T,
        ) -> impl FnMut(&mut S, &mut Services, &Ctx) -> Result<Option<R>, E>
        where
            E: Into<Error>,
            A: Extract<S, Error = E> + Send + Sync,
            T: Store<S, O, Ret = R, Error = E> + Send + Sync + 'static,
            H: Handler<S, A, Output = Option<O>, Error = E> + Send + Sync + 'static,
        {
            Box::new(move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
                let val = handler.invoke(set, ser, A::extract(set, ser, ctx)?)?;
                let arg = ctx.arg();
                let arg = arg.as_ref().map(|v| v.as_ref());
                let uid = ctx.uid();

                store.process(uid, set, ser, arg, val)
            })
        }

    }
    else {
        /// Wrap the handler and call the default action of option if return value is `Some()`,
        /// otherwise call the [`fallback`](crate::ser::InvokeService::fallback).
        pub fn wrap_handler_fallback<S, A, O, H, E>(
            mut handler: H,
        ) -> impl FnMut(&mut S, &mut Services, &Ctx) -> Result<Option<()>, Error>
        where
            O: 'static,
            S: Set,
            SetOpt<S>: Opt,
            E: Into<Error>,
            A: Extract<S, Error = E>,
            H: Handler<S, A, Output = Option<O>, Error = E> + 'static,
        {
            move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
                let val = handler
                    .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                    .map_err(Into::into)?;

                if val.is_some() {
                    let arg = ctx.arg();
                    let arg = arg.as_ref().map(|v| v.as_ref());
                    let uid = ctx.uid();
                    let mut act = *set.opt(uid)?.action();

                    act.process(uid, set, ser, arg, val)
                } else {
                    Invoker::fallback(set, ser, ctx)
                }
            }
        }

        /// Wrap the handler and call the default action of option.
        pub fn wrap_handler_action<S, A, O, H, E>(
            mut handler: H,
        ) -> impl FnMut(&mut S, &mut Services, &Ctx) -> Result<Option<()>, Error>
        where
            O: 'static,
            S: Set,
            SetOpt<S>: Opt,
            E: Into<Error>,
            A: Extract<S, Error = E>,
            H: Handler<S, A, Output = Option<O>, Error = E> + 'static,
        {
            move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
                let val = handler
                    .invoke(set, ser, A::extract(set, ser, ctx).map_err(Into::into)?)
                    .map_err(Into::into)?;
                let arg = ctx.arg();
                let arg = arg.as_ref().map(|v| v.as_ref());
                let uid = ctx.uid();
                let mut act = *set.opt(uid)?.action();

                act.process(uid, set, ser, arg, val)
            }
        }

        /// Wrap the handler and call the [`process`](Store::process) of given `store` on return value of `handler`.
        pub fn wrap_handler<S, A, O, R, H, T, E>(
            mut handler: H,
            mut store: T,
        ) -> impl FnMut(&mut S, &mut Services, &Ctx) -> Result<Option<R>, E>
        where
            E: Into<Error>,
            A: Extract<S, Error = E>,
            T: Store<S, O, Ret = R, Error = E> + 'static,
            H: Handler<S, A, Output = Option<O>, Error = E> + 'static,
        {
            Box::new(move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
                let val = handler.invoke(set, ser, A::extract(set, ser, ctx)?)?;
                let arg = ctx.arg();
                let arg = arg.as_ref().map(|v| v.as_ref());
                let uid = ctx.uid();

                store.process(uid, set, ser, arg, val)
            })
        }

    }
}
