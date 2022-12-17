pub(crate) mod context;
pub(crate) mod extract;
pub(crate) mod handler;

pub use self::context::Ctx;
pub use self::extract::Extract;
pub use self::handler::Handler;

use crate::ext::ServicesExt;
use crate::opt::Opt;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::set::Set;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::Error;
use crate::RawVal;
use crate::Uid;

/// The [`Store`] processer save the value of given option into
/// [`ValServices`](crate::ser::ValService) and [`RawValServices`](crate::ser::RawValService).
pub trait Store<Set, Value> {
    type Ret;
    type Error: Into<Error>;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error>;
}

impl<Func, Set, Value, Ret, Err> Store<Set, Value> for Func
where
    Err: Into<Error>,
    Func: FnMut(
        Uid,
        &mut Set,
        &mut Services,
        Option<&RawVal>,
        Option<Value>,
    ) -> Result<Option<Ret>, Err>,
{
    type Ret = Ret;
    type Error = Err;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        (self)(uid, set, ser, raw, val)
    }
}

/// Null store, do nothing. See [`Action`](crate::opt::Action) for default store.
pub struct NullStore;

impl<Set, Value> Store<Set, Value> for NullStore {
    type Ret = Value;

    type Error = Error;

    fn process(
        &mut self,
        _: Uid,
        _: &mut Set,
        _: &mut Services,
        _: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        Ok(val)
    }
}

/// Vector store, append the value to the [`ValService`](crate::ser::ValService)
/// if option's action is Action::App.
/// See [`Action`](crate::opt::Action) for default store.
pub struct VecStore;

impl<Set, Value: 'static> Store<Set, Vec<Value>> for VecStore
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
{
    type Ret = ();

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Vec<Value>>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let has_value = val.is_some();

        // Set the value if return Some(Value)
        if let Some(val) = val {
            let raw_ser = ser.ser_rawval_mut()?;

            if let Some(raw) = raw {
                raw_ser.push(uid, raw.clone());
            }

            let val_ser = ser.ser_val_mut()?;

            if let Some(opt) = set.get(uid) {
                if opt.action().is_app() {
                    for value in val {
                        val_ser.push(uid, value);
                    }
                }
            }
        }

        Ok(has_value.then_some(()))
    }
}

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
            InvokeService::fallback(set, ser, ctx)
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
