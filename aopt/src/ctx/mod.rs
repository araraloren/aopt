pub(crate) mod ctx;
pub(crate) mod extract;
pub(crate) mod handler;

pub use self::ctx::Ctx;
pub use self::extract::ExtractCtx;
pub use self::handler::Handler;

use std::fmt::Debug;

use crate::ser::Services;
use crate::Error;
use crate::RawVal;
use crate::Uid;

/// The callback used in [`InvokeService`](crate::ser::InvokeService`).
pub trait Callback<Set> {
    type Value;
    type Error: Into<Error>;

    fn invoke(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<Self::Value>, Self::Error>;
}

impl<Func, Set, Value, Err> Callback<Set> for Func
where
    Err: Into<Error>,
    Func: FnMut(Uid, &mut Set, &mut Services, &Ctx) -> Result<Option<Value>, Err>,
{
    type Value = Value;
    type Error = Err;

    fn invoke(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<Self::Value>, Self::Error> {
        (self)(uid, set, ser, ctx)
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

/// Wrap only the handler, user can custom the value store logical.
pub fn wrap_handler<Set, Args, Ret, Error>(
    mut handler: impl Handler<Set, Args, Output = Option<Ret>, Error = Error> + 'static,
) -> Callbacks<Set, Ret, Error>
where
    Error: Into<crate::Error>,
    Args: ExtractCtx<Set, Error = Error>,
{
    Box::new(
        move |uid: Uid, set: &mut Set, ser: &mut Services, ctx: &Ctx| {
            let val = handler.invoke(uid, set, Args::extract(uid, set, ser, ctx)?)?;
            Ok(val)
        },
    )
}

/// Wrap the handler and store.
pub fn wrap_storer<Set, Args, Output, Ret, Error>(
    mut handler: impl Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    mut store: impl Store<Set, Output, Ret = Ret, Error = Error> + 'static,
) -> Callbacks<Set, Ret, Error>
where
    Error: Into<crate::Error>,
    Args: ExtractCtx<Set, Error = Error>,
{
    Box::new(
        move |uid: Uid, set: &mut Set, ser: &mut Services, ctx: &Ctx| {
            let val = handler.invoke(uid, set, Args::extract(uid, set, ser, ctx)?)?;

            Ok(store.process(uid, set, ser, ctx.arg(), val)?)
        },
    )
}
