use super::ExtractFromCtx;
use super::Handler;
use super::Services;
use crate::astr;
use crate::ctx::wrap_callback;
use crate::ctx::Callbacks;
use crate::ctx::Context;
use crate::opt::Opt;
use crate::ser::Service;
use crate::set::Set;
use crate::Error;
use crate::HashMap;
use crate::Str;
use crate::Uid;

/// Save the callback with key [`Uid`].
///
/// # Example
/// ```rust
/// # use aopt_stable::aopt::UserData;
/// # use aopt_stable::prelude::*;
/// # use aopt_stable::Error;
/// # use aopt_stable::Result;
/// #
/// pub struct Arg(Str);
///
/// // implement ExtractFromCtx for your type
/// impl ExtractFromCtx<SimpleSet> for Arg {
///     type Error = Error;
///
///     fn extract_from(_uid: Uid, _set: &SimpleSet, _ser: &mut Services, ctx: Context) -> Result<Self> {
///         Ok(Arg(ctx.get_argument().unwrap_or_default()))
///     }
/// }
///
/// fn main() -> Result<()> {
///     let mut is = InvokeService::<SimpleSet, Str>::new();
///
///     // you can register callback into InvokeService
///     is.register(0, |_uid: Uid, _set: &mut SimpleSet| Ok(None));
///     is.register(0, |_uid: Uid, _set: &mut SimpleSet, arg: Arg| {
///         Ok(Some(arg.0.clone()))
///     });
///     is.register(0, |_uid: Uid, _set: &mut SimpleSet, data: UserData<i64>| {
///         Ok(Some(Str::from(data.to_string())))
///     });
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct InvokeService<S, V>
where
    S: Set,
{
    callbacks: HashMap<Uid, Callbacks<S, V, Error>>,
}

impl<S, V> InvokeService<S, V>
where
    S: Set,
    V: From<Str>,
{
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }

    pub fn register_raw(&mut self, uid: Uid, handler: Callbacks<S, V, Error>) -> &mut Self {
        self.callbacks.insert(uid, handler);
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    pub fn register<H, Args>(&mut self, uid: Uid, handler: H) -> &mut Self
    where
        Args: ExtractFromCtx<S, Error = Error> + 'static,
        H: Handler<S, Args, Output = Option<V>, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_callback(handler));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<S, V> InvokeService<S, V>
where
    S: Set,
    V: From<Str>,
    S::Opt: Opt,
{
    /// Invoke the callback saved in [`InvokeService`], return None if the callback not exist.
    pub fn invoke(
        &mut self,
        uid: Uid,
        set: &mut S,
        ser: &mut Services,
        ctx: Context,
    ) -> Result<Option<V>, Error> {
        if let Some(callback) = self.callbacks.get_mut(&uid) {
            Ok(callback.invoke(uid, set, ser, ctx)?)
        } else {
            Ok(None)
        }
    }
}

impl<S, V> Service for InvokeService<S, V>
where
    S: Set,
{
    fn service_name() -> Str {
        astr("InvokeService")
    }
}