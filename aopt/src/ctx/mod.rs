pub(crate) mod ctx;
pub(crate) mod data;
pub(crate) mod extract;
pub(crate) mod handler;

pub use self::ctx::Ctx;
pub use self::data::Data;
pub use self::extract::ExtractCtx;
pub use self::handler::Handler;

use std::fmt::Debug;

use crate::ser::Services;
use crate::Error;
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
        ctx: Ctx,
    ) -> Result<Option<Self::Value>, Self::Error>;
}

impl<Func, Set, Value, Err> Callback<Set> for Func
where
    Err: Into<Error>,
    Func: FnMut(Uid, &mut Set, &mut Services, Ctx) -> Result<Option<Value>, Err>,
{
    type Value = Value;
    type Error = Err;

    fn invoke(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: Ctx,
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

/// Wrap a function and return [`Callbacks`].
///
/// # Examples
///
/// [`InvokeService`](crate::ser::InvokeService`) will use [`wrap_callback`] wrap the handler pass to `register`.
///
/// ```no_run
/// todo!()
/// ```
pub fn wrap_callback<Hanlder, Set, Args, Value, Error>(
    mut handler: Hanlder,
) -> Callbacks<Set, Value, Error>
where
    Error: Into<crate::Error>,
    Hanlder::Output: Into<Option<Value>>, // Callbacks' invoke returns Option<Value>
    Args: ExtractCtx<Set, Error = Error>,
    Hanlder: self::handler::Handler<Set, Args, Error = Error> + 'static,
{
    Box::new(
        move |uid: Uid, set: &mut Set, ser: &mut Services, ctx: Ctx| {
            Ok(handler
                .invoke(uid, set, Args::extract(uid, set, ser, &ctx)?)?
                .into())
        },
    )
}
