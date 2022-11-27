pub(crate) mod context;
pub(crate) mod extract;
pub(crate) mod handler;

pub use self::context::Ctx;
pub use self::extract::Extract;
pub use self::handler::Handler;

use std::fmt::Debug;

use crate::opt::Opt;
use crate::ser::Services;
use crate::set::Set;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::Error;
use crate::RawVal;
use crate::Uid;

/// The callback used in [`InvokeService`](crate::ser::InvokeService`).
pub trait Callback<Set> {
    type Value;
    type Error: Into<Error>;

    fn invoke(
        &mut self,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<Self::Value>, Self::Error>;
}

impl<Func, Set, Value, Err> Callback<Set> for Func
where
    Err: Into<Error>,
    Func: FnMut(&mut Set, &mut Services, &Ctx) -> Result<Option<Value>, Err>,
{
    type Value = Value;
    type Error = Err;

    fn invoke(
        &mut self,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<Self::Value>, Self::Error> {
        (self)(set, ser, ctx)
    }
}

/// The callback create by user should return `Option<Ret>`.
pub type Callbacks<Set, Value, Error> = Box<dyn Callback<Set, Value = Value, Error = Error>>;

impl<Set, Value, Error> Debug for Callbacks<Set, Value, Error> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Box")
            .field(&"dyn Callback".to_string())
            .finish()
    }
}

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

/// Wrap the handler and call the default action of option.
pub fn wrap_handler_default<S, A, O>(
    mut handler: impl Handler<S, A, Output = Option<O>, Error = Error> + 'static,
) -> Callbacks<S, (), Error>
where
    O: 'static,
    S: Set,
    SetOpt<S>: Opt,
    A: Extract<S, Error = Error>,
{
    Box::new(move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
        let val = handler.invoke(set, ser, A::extract(set, ser, ctx)?)?;
        let arg = ctx.arg();
        let arg = arg.as_ref().map(|v| v.as_ref());
        let uid = ctx.uid();
        let mut act = *set.opt(uid)?.action();

        act.process(uid, set, ser, arg, val)
    })
}

/// Wrap the handler and call the [`process`](Store::process) of given `store` on return value of `handler`.
pub fn wrap_handler<S, A, O, R, E>(
    mut handler: impl Handler<S, A, Output = Option<O>, Error = E> + 'static,
    mut store: impl Store<S, O, Ret = R, Error = E> + 'static,
) -> Callbacks<S, R, E>
where
    E: Into<crate::Error>,
    A: Extract<S, Error = E>,
{
    Box::new(move |set: &mut S, ser: &mut Services, ctx: &Ctx| {
        let val = handler.invoke(set, ser, A::extract(set, ser, ctx)?)?;
        let arg = ctx.arg();
        let arg = arg.as_ref().map(|v| v.as_ref());
        let uid = ctx.uid();

        store.process(uid, set, ser, arg, val)
    })
}
