use std::fmt::Debug;

use super::ExtractCtx;
use super::Handler;
use super::Services;
use crate::astr;
use crate::ctx::wrap_handler;
use crate::ctx::wrap_handler_serde;
use crate::ctx::Callbacks;
use crate::ctx::Ctx;
use crate::ctx::Serializer;
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
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use aopt::Result;
/// # use ctx::Data;
/// #
/// pub struct Arg(Str);
///
/// // implement ExtractCtx for your type
/// impl ExtractCtx<SimSet> for Arg {
///     type Error = Error;
///
///     fn extract(_uid: Uid, _set: &SimSet, _ser: &Services, ctx: &Ctx) -> Result<Self> {
///         Ok(Arg(ctx.arg().cloned().unwrap_or_default()))
///     }
/// }
///
/// fn main() -> Result<()> {
///     let mut is = InvokeService::<SimSet, Str>::new();
///
///     // you can register callback into InvokeService
///     is.reg(0, |_uid: Uid, _set: &mut SimSet| Ok(None));
///     is.reg(0, |_uid: Uid, _set: &mut SimSet, arg: Arg| {
///         Ok(Some(arg.0.clone()))
///     });
///     is.reg(0, |_uid: Uid, _set: &mut SimSet, data: Data<i64>| {
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
    pub fn reg_raw(&mut self, uid: Uid, handler: Callbacks<Set, Value, Error>) -> &mut Self {
        self.callbacks.insert(uid, handler);
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    pub fn reg<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Output, Error = Error> + 'static,
    ) -> &mut Self
    where
        Output: Into<Option<Value>>,
        Args: ExtractCtx<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_handler(handler));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    pub fn reg_serde<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Output, Error = Error> + 'static,
        serializer: impl Serializer<Output = Option<Value>, Error = Error> + 'static,
    ) -> &mut Self
    where
        Output: serde::Serialize,
        Args: ExtractCtx<Set, Error = Error> + 'static,
    {
        self.callbacks
            .insert(uid, wrap_handler_serde(handler, serializer));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<Set, Value> InvokeService<Set, Value>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
{
    /// Invoke the callback saved in [`InvokeService`], return None if the callback not exist.
    pub fn invoke(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: Ctx,
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
