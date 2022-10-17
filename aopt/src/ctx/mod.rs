pub(crate) mod context;
pub(crate) mod data;
pub(crate) mod extract;
pub(crate) mod handler;
pub(crate) mod value;

pub use self::context::Ctx;
pub use self::context::CtxDisbale;
pub use self::context::CtxIdx;
pub use self::context::CtxLen;
pub use self::context::CtxMatArg;
pub use self::context::CtxOptName;
pub use self::context::CtxPrefix;
pub use self::context::CtxStyle;
pub use self::context::CtxUid;
pub use self::data::Data;
pub use self::extract::ExtractCtx;
pub use self::handler::Handler;
pub use self::value::Value;

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

/// Wrap a function and return [`Callbacks`].
///
/// # Note
/// ```txt
///  _______________________________________________________________________
/// |   |Uid, &mut Set, &mut Services, { Other Args }| -> T: Into<Option<Value>>
///         |
///      wrapped
///         |
///         v
///  ___________________________________________________________________
/// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<Value>
///         |
///      invoked
///         |
///         v
///  ______________________________________________________
/// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
/// |       call Handler::invoke(&mut self, Uid, &mut Set, Args)
/// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
/// |           -> T: Into<Option<Value>>
/// |       -> Option<Value>
/// ```
/// # Examples
///
/// [`InvokeService`](crate::ser::InvokeService`) will use [`wrap_handler`] wrap the handler pass to `register`.
/// ```no_run
/// todo!()
/// ```
pub fn wrap_handler<Set, Args, Output, Value, Error>(
    mut handler: impl Handler<Set, Args, Output = Output, Error = Error> + 'static,
) -> Callbacks<Set, Value, Error>
where
    Error: Into<crate::Error>,
    Output: Into<Option<Value>>, // Callbacks' invoke returns Option<Value>
    Args: ExtractCtx<Set, Error = Error>,
{
    Box::new(
        move |uid: Uid, set: &mut Set, ser: &mut Services, ctx: &Ctx| {
            Ok(handler
                .invoke(uid, set, Args::extract(uid, set, ser, ctx)?)?
                .into())
        },
    )
}

pub trait Serializer {
    type Output;
    type Error: Into<Error>;

    fn serialize<S: serde::Serialize>(&mut self, value: S) -> Result<Self::Output, Self::Error>;
}

/// Wrap a function and return [`Callbacks`].
///
/// # Note
/// ```txt
///  _______________________________________________________________________
/// |   |Uid, &mut Set, &mut Services, { Other Args }| -> T: Into<Option<Value>>
///         |
///      wrapped
///         |
///         v
///  ___________________________________________________________________
/// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<Value>
///         |
///      invoked
///         |
///         v
///  ______________________________________________________
/// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
/// |       call Handler::invoke(&mut self, Uid, &mut Set, Args)
/// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
/// |       --> T: serde::Serialize
/// |       call Serializer::serialize<S: serde::Serialize>(&mut self, S)
/// |       -> Option<Value>
/// ```
pub fn wrap_handler_serde<Set, Args, Output, Value, Error>(
    mut handler: impl Handler<Set, Args, Output = Output, Error = Error> + 'static,
    mut serializer: impl Serializer<Output = Option<Value>, Error = Error> + 'static,
) -> Callbacks<Set, Value, Error>
where
    Error: Into<crate::Error>,
    Output: serde::Serialize,
    Args: ExtractCtx<Set, Error = Error>,
{
    Box::new(
        move |uid: Uid, set: &mut Set, ser: &mut Services, ctx: &Ctx| {
            let value: Output = handler.invoke(uid, set, Args::extract(uid, set, ser, ctx)?)?;

            Ok(serializer.serialize(value)?)
        },
    )
}
