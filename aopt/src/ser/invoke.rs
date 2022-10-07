use std::fmt::Debug;

use super::ExtractFromCtx;
use super::Handler;
use super::Services;
use crate::astr;
use crate::ctx::wrap_callback;
use crate::ctx::Callbacks;
use crate::ctx::Context;
use crate::opt::Opt;
use crate::ser::Service;
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
pub struct InvokeService<Set, Value> {
    callbacks: HashMap<Uid, Callbacks<Set, Value, Error>>,
}

impl<Set, Value> Debug for InvokeService<Set, Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InvokeService")
            .field("callbacks", &self.callbacks)
            .finish()
    }
}

impl<Set, Value> Default for InvokeService<Set, Value> {
    fn default() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set, Value> InvokeService<Set, Value> {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set, Value> InvokeService<Set, Value> {
    pub fn register_raw(&mut self, uid: Uid, handler: Callbacks<Set, Value, Error>) -> &mut Self {
        self.callbacks.insert(uid, handler);
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    pub fn register<H, Args>(&mut self, uid: Uid, handler: H) -> &mut Self
    where
        Args: ExtractFromCtx<Set, Error = Error> + 'static,
        H: Handler<Set, Args, Output = Option<Value>, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_callback(handler));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<Set, Value> InvokeService<Set, Value>
where
    Set: crate::set::Set,
    Value: From<Str>,
    Set::Opt: Opt,
{
    /// Invoke the callback saved in [`InvokeService`], return None if the callback not exist.
    pub fn invoke(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: Context,
    ) -> Result<Option<Value>, Error> {
        if let Some(callback) = self.callbacks.get_mut(&uid) {
            Ok(callback.invoke(uid, set, ser, ctx)?)
        } else {
            Ok(None)
        }
    }
}

impl<S, V> Service for InvokeService<S, V> {
    fn service_name() -> Str {
        astr("InvokeService")
    }
}
