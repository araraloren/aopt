use std::fmt::Debug;

use super::ExtractCtx;
use super::Handler;
use super::Services;
use crate::astr;
use crate::ctx::wrap_handler;
use crate::ctx::wrap_handler2;
use crate::ctx::wrap_handler_serde;
use crate::ctx::Callbacks;
use crate::ctx::Ctx;
use crate::ctx::Serializer;
use crate::opt::Opt;
use crate::opt::value::Value;
use crate::prelude::ValType;
use crate::ser::Service;
use crate::Error;
use crate::HashMap;
use crate::Str;
use crate::Uid;
use crate::opt::RawValParser;

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
pub struct InvokeService<Set, Ret = ()> {
    callbacks: HashMap<Uid, Callbacks<Set, Ret, Error>>,
}

impl<Set, Ret> Debug for InvokeService<Set, Ret> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InvokeService")
            .field("callbacks", &self.callbacks)
            .finish()
    }
}

impl<Set, Ret> Default for InvokeService<Set, Ret> {
    fn default() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set, Ret> InvokeService<Set, Ret> {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set, Ret> InvokeService<Set, Ret> {
    pub fn reg_raw(&mut self, uid: Uid, handler: Callbacks<Set, Ret, Error>) -> &mut Self {
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
        Output: Into<Option<Ret>>,
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
        serializer: impl Serializer<Output = Option<Ret>, Error = Error> + 'static,
    ) -> &mut Self
    where
        Output: serde::Serialize,
        Args: ExtractCtx<Set, Error = Error> + 'static,
    {
        self.callbacks
            .insert(uid, wrap_handler_serde(handler, serializer));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    pub fn reg2<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Output, Error = Error> + 'static,
        store: impl crate::ctx::Store<Set, Output, Ret = Ret, Error = Error> + 'static,
    ) -> &mut Self
    where
        Output: Into<Option<Ret>>,
        Args: ExtractCtx<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_handler2(handler, store));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<Set, Ret> InvokeService<Set, Ret>
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
        ctx: &Ctx,
    ) -> Result<Option<Ret>, Error> {
        if let Some(callback) = self.callbacks.get_mut(&uid) {
            Ok(callback.invoke(uid, set, ser, ctx)?)
        } else {
            Ok(None)
        }
    }

    pub fn invoke_default(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<()>, Error> {
        let opt = set.get(uid).unwrap();
        let val_ty = opt.val_ty();
        
        if val_ty == ValType::Int {
            let mut parser = Value(0);
            let val: i32 = parser.parse(opt, ctx.arg().cloned(), ctx)?;
        }
        Ok(Some(()))
    }
}

impl<S, V> Service for InvokeService<S, V> {
    fn service_name() -> Str {
        astr("InvokeService")
    }
}
