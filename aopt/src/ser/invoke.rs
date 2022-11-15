use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::trace;

use crate::astr;
use crate::ctx::wrap_handler;
use crate::ctx::wrap_handler_store;
use crate::ctx::wrap_serhandler;
use crate::ctx::wrap_serhandler_store;
use crate::ctx::Callbacks;
use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::SerHandler;
use crate::ctx::Store;
use crate::opt::Opt;
use crate::opt::RawValParser;
use crate::opt::ValAssoc;
use crate::opt::ValStore;
use crate::ser::Service;
use crate::ser::Services;
use crate::Error;
use crate::HashMap;
use crate::Str;
use crate::Uid;

/// Keep the variable length arguments handler in [`HashMap`] with key [`Uid`].
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use aopt::Arc;
/// # use aopt::RawVal;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
///    pub struct Count(usize);
///
///    // implement Extract for your type
///    impl Extract<ASet> for Count {
///        type Error = Error;
///
///        fn extract(_uid: Uid, _set: &ASet, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
///            Ok(Self(ctx.args().len()))
///        }
///    }
///    let mut ser = Services::default().with(UsrValService::default());
///    let mut is = InvokeService::new();
///    let mut set = ASet::default();
///    let args = Arc::new(Args::new(["--foo", "bar", "doo"].into_iter()));
///    let ctx = Ctx::default().with_args(args);
///
///    ser.ser_data_mut()?.insert(ser::Value::new(42i64));
///    // you can register callback into InvokeService
///    is.register(
///        0,
///        |uid: Uid, _set: &mut ASet| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{{uid}}}");
///            Ok(None)
///        },
///    )
///    .with_default();
///    is.register(
///        1,
///        |uid: Uid, _set: &mut ASet, cnt: Count| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{{uid}}}");
///            assert_eq!(cnt.0, 3);
///            Ok(None)
///        },
///    )
///    .with_default();
///    is.register(
///        2,
///        |uid: Uid, _set: &mut ASet, data: ser::Value<i64>| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{{uid}}}");
///            assert_eq!(data.as_ref(), &42);
///            Ok(None)
///        },
///    )
///    .with_default();
///
///    is.invoke(0, &mut set, &mut ser, &ctx)?;
///    is.invoke(1, &mut set, &mut ser, &ctx)?;
///    is.invoke(2, &mut set, &mut ser, &ctx)?;
/// #
/// #   Ok(())
/// # }
/// ```
pub struct InvokeService<Set> {
    callbacks: HashMap<Uid, Callbacks<Set, (), Error>>,
}

impl<Set> Debug for InvokeService<Set> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InvokeService")
            .field("callbacks", &self.callbacks)
            .finish()
    }
}

impl<Set> Default for InvokeService<Set> {
    fn default() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set> InvokeService<Set> {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set> InvokeService<Set> {
    pub fn set_raw(&mut self, uid: Uid, handler: Callbacks<Set, (), Error>) -> &mut Self {
        self.callbacks.insert(uid, handler);
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    ///
    /// The [`InvokeService`]  will call the [`invoke`](crate::ctx::Handler::invoke).
    /// # Note
    /// ```txt
    /// |   handler: |Uid, &mut Set, &mut Services, { Other Args }| -> Result<Option<()>, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<()>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
    /// |       call Handler::invoke(&mut self, Uid, &mut Set, &mut Services, Args)
    /// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
    /// |           -> Result<Option<()>, Error>
    /// ```
    pub fn set_serhandler<Args>(
        &mut self,
        uid: Uid,
        handler: impl SerHandler<Set, Args, Output = Option<()>, Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: Extract<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_serhandler(handler));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    ///
    /// The [`InvokeService`]  will call the [`invoke`](crate::ctx::Handler::invoke).
    /// # Note
    /// ```txt
    /// |   handler: |Uid, &mut Set, &mut Services, { Other Args }| -> Result<Option<()>, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<()>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
    /// |       call Handler::invoke(&mut self, Uid, &mut Set, Args)
    /// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
    /// |           -> Result<Option<()>, Error>
    /// ```
    pub fn set_handler<Args>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Option<()>, Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: Extract<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_handler(handler));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    ///
    /// The [`InvokeService`] first call the [`invoke`](crate::ctx::Handler::invoke), then
    /// call the [`process`](crate::ctx::Store::process) with the return value.
    /// # Note
    /// ```txt
    /// |   handler: |Uid, &mut Set, &mut Services, { Other Args }| -> Result<Option<Value>, Error>
    /// |   storer: |Uid, &mut Set, &mut Services, Option<&RawVal>, Option<Value>| -> Result<Option<()>, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<Value>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
    /// |       call Handler::invoke(&mut self, Uid, &mut Set, &mut Services, Args)
    /// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
    /// |           -> Result<Option<Value>, Error>
    /// |       -> call Store::process(Uid, &Set, Option<&RawVal>, Option<Value>)
    /// |           -> Result<Option<()>, Error>
    /// ```
    pub fn set_serhandler_store<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        store: impl Store<Set, Output, Ret = (), Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: Extract<Set, Error = Error> + 'static,
    {
        self.callbacks
            .insert(uid, wrap_serhandler_store(handler, store));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    ///
    /// The [`InvokeService`] first call the [`invoke`](crate::ctx::Handler::invoke), then
    /// call the [`process`](crate::ctx::Store::process) with the return value.
    /// # Note
    /// ```txt
    /// |   handler: |Uid, &mut Set, &mut Services, { Other Args }| -> Result<Option<Value>, Error>
    /// |   storer: |Uid, &mut Set, &mut Services, Option<&RawVal>, Option<Value>| -> Result<Option<()>, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<Value>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
    /// |       call Handler::invoke(&mut self, Uid, &mut Set, Args)
    /// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
    /// |           -> Result<Option<Value>, Error>
    /// |       -> call Store::process(Uid, &Set, Option<&RawVal>, Option<Value>)
    /// |           -> Result<Option<()>, Error>
    /// ```
    pub fn set_handler_store<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        store: impl Store<Set, Output, Ret = (), Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: Extract<Set, Error = Error> + 'static,
    {
        self.callbacks
            .insert(uid, wrap_handler_store(handler, store));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<Set> InvokeService<Set>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
{
    pub fn register<Args, Output, H>(
        &mut self,
        uid: Uid,
        handler: H,
    ) -> Register<'_, Set, H, Args, Output>
    where
        H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        Args: Extract<Set, Error = Error> + 'static,
    {
        Register {
            ser: self,
            handler: Some(handler),
            register: false,
            uid,
            marker: PhantomData::default(),
        }
    }

    pub fn register_ser<Args, Output, H>(
        &mut self,
        uid: Uid,
        handler: H,
    ) -> SerRegister<'_, Set, H, Args, Output>
    where
        H: SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        Args: Extract<Set, Error = Error> + 'static,
    {
        SerRegister {
            ser: self,
            handler: Some(handler),
            register: false,
            uid,
            marker: PhantomData::default(),
        }
    }
}

impl<Set> InvokeService<Set>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
{
    /// Invoke the handler saved in [`InvokeService`], it will panic if the handler not exist.
    pub fn invoke(
        &mut self,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<()>, Error> {
        let uid = ctx.uid();
        if let Some(callback) = self.callbacks.get_mut(&uid) {
            return Ok(callback.invoke(set, ser, ctx)?);
        }
        unreachable!(
            "There is no callback of {}, call `invoke_default` instead",
            uid
        )
    }

    /// Invoke the default option handler of [`InvokeService`].
    ///
    /// The default handler will parsing the argument into associated type value,
    /// then save the value to [`ValService`] through default [`ValStore`].
    pub fn invoke_default(
        &mut self,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<()>, Error> {
        let uid = ctx.uid();
        let opt = set.get(uid).unwrap();
        let assoc = opt.assoc();
        let arg = ctx.arg();
        let val = arg.as_ref().map(|v| v.as_ref());
        let mut store = ValStore::default();

        trace!("Invoke default handler for {{{uid}}}, ctx{{{ctx:?}}}");
        match assoc {
            ValAssoc::Bool => store.process(uid, set, ser, val, bool::parse(opt, val, ctx).ok()),
            ValAssoc::Int => store.process(uid, set, ser, val, i64::parse(opt, val, ctx).ok()),
            ValAssoc::Uint => store.process(uid, set, ser, val, u64::parse(opt, val, ctx).ok()),
            ValAssoc::Flt => store.process(uid, set, ser, val, f64::parse(opt, val, ctx).ok()),
            ValAssoc::Str => store.process(uid, set, ser, val, String::parse(opt, val, ctx).ok()),
            ValAssoc::Noa => store.process(uid, set, ser, val, val.map(|_| true)),
            ValAssoc::Null => Ok(Some(())),
        }
    }
}

impl<Set> Service for InvokeService<Set> {
    fn service_name() -> Str {
        astr("InvokeService")
    }
}

pub struct SerRegister<'a, Set, Handler, Args, Output> {
    ser: &'a mut InvokeService<Set>,

    handler: Option<Handler>,

    register: bool,

    uid: Uid,

    marker: PhantomData<(Args, Output)>,
}

impl<'a, Args, Set, Output, H> SerRegister<'a, Set, H, Args, Output>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
    H: SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: Extract<Set, Error = Error> + 'static,
{
    /// Register the handler with given [`Store`] implementation.
    pub fn with(&mut self, store: impl Store<Set, Output, Ret = (), Error = Error> + 'static) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser.set_serhandler_store(self.uid, handler, store);
            self.register = true;
        }
    }
}

impl<'a, Args, Set, Output, H> SerRegister<'a, Set, H, Args, Output>
where
    Output: 'static,
    Set: crate::set::Set,
    Set::Opt: Opt,
    H: SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: Extract<Set, Error = Error> + 'static,
{
    /// Register the handler with default [`ValStore`].
    pub fn with_default(&mut self) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser
                .set_serhandler_store(self.uid, handler, ValStore::default());
            self.register = true;
        }
    }
}

impl<'a, Set, Handler, Args, Output> Drop for SerRegister<'a, Set, Handler, Args, Output> {
    fn drop(&mut self) {
        if !self.register {
            panic!("Consider call with_default or with on Register")
        }
    }
}

pub struct Register<'a, Set, Handler, Args, Output> {
    ser: &'a mut InvokeService<Set>,

    handler: Option<Handler>,

    register: bool,

    uid: Uid,

    marker: PhantomData<(Args, Output)>,
}

impl<'a, Args, Set, Output, H> Register<'a, Set, H, Args, Output>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
    H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: Extract<Set, Error = Error> + 'static,
{
    /// Register the handler with given [`Store`] implementation.
    pub fn with(&mut self, store: impl Store<Set, Output, Ret = (), Error = Error> + 'static) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser.set_handler_store(self.uid, handler, store);
            self.register = true;
        }
    }
}

impl<'a, Args, Set, Output, H> Register<'a, Set, H, Args, Output>
where
    Output: 'static,
    Set: crate::set::Set,
    Set::Opt: Opt,
    H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: Extract<Set, Error = Error> + 'static,
{
    /// Register the handler with default [`ValStore`].
    pub fn with_default(&mut self) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser
                .set_handler_store(self.uid, handler, ValStore::default());
            self.register = true;
        }
    }
}

impl<'a, Set, Handler, Args, Output> Drop for Register<'a, Set, Handler, Args, Output> {
    fn drop(&mut self) {
        if !self.register {
            panic!("Consider call with_default or with on Register")
        }
    }
}
