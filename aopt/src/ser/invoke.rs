use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::trace;

use crate::astr;
use crate::ctx::wrap_handler;
use crate::ctx::wrap_ser_handler;
use crate::ctx::wrap_ser_store;
use crate::ctx::wrap_store;
use crate::ctx::Callbacks;
use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
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
///    // implement ExtractCtx for your type
///    impl ExtractCtx<ASet> for Count {
///        type Error = Error;
///
///        fn extract(_uid: Uid, _set: &ASet, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
///            Ok(Self(ctx.args().len()))
///        }
///    }
///    let mut ser = Services::default().with(DataService::default());
///    let mut is = InvokeService::<ASet>::new();
///    let mut set = ASet::default();
///    let args = Arc::new(Args::new(["--foo", "bar", "doo"].into_iter()));
///    let ctx = Ctx::default().with_args(args);
///
///    ser.ser_data_mut()?.insert(ser::Data::new(42i64));
///    // you can register callback into InvokeService
///    is.register(
///        0,
///        |uid: Uid, _set: &mut ASet| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{{uid}}}");
///            Ok(None)
///        },
///    )
///    .or_default();
///    is.register(
///        1,
///        |uid: Uid, _set: &mut ASet, cnt: Count| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{{uid}}}");
///            assert_eq!(cnt.0, 3);
///            Ok(None)
///        },
///    )
///    .or_default();
///    is.register(
///        2,
///        |uid: Uid, _set: &mut ASet, data: ser::Data<i64>| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{{uid}}}");
///            assert_eq!(data.as_ref(), &42);
///            Ok(None)
///        },
///    )
///    .or_default();
///
///    is.invoke(0, &mut set, &mut ser, &ctx)?;
///    is.invoke(1, &mut set, &mut ser, &ctx)?;
///    is.invoke(2, &mut set, &mut ser, &ctx)?;
/// #
/// #   Ok(())
/// # }
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
    pub fn register_raw(&mut self, uid: Uid, handler: Callbacks<Set, Ret, Error>) -> &mut Self {
        self.callbacks.insert(uid, handler);
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    ///
    /// The [`InvokeService`]  will call the [`invoke`](crate::ctx::Handler::invoke).
    /// # Note
    /// ```txt
    /// |   handler: |Uid, &mut Set, &mut Services, { Other Args }| -> Result<Option<Ret>, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<Ret>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
    /// |       call Handler::invoke(&mut self, Uid, &mut Set, &mut Services, Args)
    /// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
    /// |           -> Result<Option<Ret>, Error>
    /// ```
    pub fn register_serhandler<Args>(
        &mut self,
        uid: Uid,
        handler: impl SerHandler<Set, Args, Output = Option<Ret>, Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: ExtractCtx<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_ser_handler(handler));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    ///
    /// The [`InvokeService`]  will call the [`invoke`](crate::ctx::Handler::invoke).
    /// # Note
    /// ```txt
    /// |   handler: |Uid, &mut Set, &mut Services, { Other Args }| -> Result<Option<Ret>, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |Uid, &mut Set, &mut Services, &Ctx| -> Option<Ret>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, Uid, &mut Set, &mut Services, &Ctx)
    /// |       call Handler::invoke(&mut self, Uid, &mut Set, Args)
    /// |           call Args::extract(Uid, &Set, &Services, &Ctx) -> Args
    /// |           -> Result<Option<Ret>, Error>
    /// ```
    pub fn register_handler<Args>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Option<Ret>, Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: ExtractCtx<Set, Error = Error> + 'static,
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
    /// |   storer: |Uid, &mut Set, &mut Services, Option<&RawVal>, Option<Value>| -> Result<Option<Ret>, Error>
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
    /// |           -> Result<Option<Ret>, Error>
    /// ```
    pub fn register_serstore<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        store: impl Store<Set, Output, Ret = Ret, Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: ExtractCtx<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_ser_store(handler, store));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::policy::Policy) when option setted.
    ///
    /// The [`InvokeService`] first call the [`invoke`](crate::ctx::Handler::invoke), then
    /// call the [`process`](crate::ctx::Store::process) with the return value.
    /// # Note
    /// ```txt
    /// |   handler: |Uid, &mut Set, &mut Services, { Other Args }| -> Result<Option<Value>, Error>
    /// |   storer: |Uid, &mut Set, &mut Services, Option<&RawVal>, Option<Value>| -> Result<Option<Ret>, Error>
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
    /// |           -> Result<Option<Ret>, Error>
    /// ```
    pub fn register_store<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        store: impl Store<Set, Output, Ret = Ret, Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: ExtractCtx<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_store(handler, store));
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
    Ret: Default + 'static,
{
    pub fn register<Args, Output, H>(
        &mut self,
        uid: Uid,
        handler: H,
    ) -> Register<'_, Set, Ret, H, Args, Output>
    where
        H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        Args: ExtractCtx<Set, Error = Error> + 'static,
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
    ) -> SerRegister<'_, Set, Ret, H, Args, Output>
    where
        H: SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        Args: ExtractCtx<Set, Error = Error> + 'static,
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

impl<Set, Ret> InvokeService<Set, Ret>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
{
    /// Invoke the handler saved in [`InvokeService`], it will panic if the handler not exist.
    pub fn invoke(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<Ret>, Error> {
        if let Some(callback) = self.callbacks.get_mut(&uid) {
            return Ok(callback.invoke(uid, set, ser, ctx)?);
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
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<()>, Error> {
        let opt = set.get(uid).unwrap();
        let assoc = opt.assoc();
        let val = ctx.arg();
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

impl<S, V> Service for InvokeService<S, V> {
    fn service_name() -> Str {
        astr("InvokeService")
    }
}

pub struct SerRegister<'a, Set, Ret, Handler, Args, Output> {
    ser: &'a mut InvokeService<Set, Ret>,

    handler: Option<Handler>,

    register: bool,

    uid: Uid,

    marker: PhantomData<(Args, Output)>,
}

impl<'a, Args, Set, Ret, Output, H> SerRegister<'a, Set, Ret, H, Args, Output>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
    Ret: Default + 'static,
    H: SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: ExtractCtx<Set, Error = Error> + 'static,
{
    /// Register the handler with given [`Store`] implementation.
    pub fn with(&mut self, store: impl Store<Set, Output, Ret = Ret, Error = Error> + 'static) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser.register_serstore(self.uid, handler, store);
            self.register = true;
        }
    }
}

impl<'a, Args, Set, Ret, Output, H> SerRegister<'a, Set, Ret, H, Args, Output>
where
    Output: 'static,
    Set: crate::set::Set,
    Set::Opt: Opt,
    Ret: Default + 'static,
    H: SerHandler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: ExtractCtx<Set, Error = Error> + 'static,
{
    /// Register the handler with default [`ValStore`].
    pub fn with_default(&mut self) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser
                .register_serstore(self.uid, handler, ValStore::default());
            self.register = true;
        }
    }
}

impl<'a, Set, Ret, Handler, Args, Output> Drop
    for SerRegister<'a, Set, Ret, Handler, Args, Output>
{
    fn drop(&mut self) {
        if !self.register {
            panic!("Consider call or_default or and_then on Register")
        }
    }
}

pub struct Register<'a, Set, Ret, Handler, Args, Output> {
    ser: &'a mut InvokeService<Set, Ret>,

    handler: Option<Handler>,

    register: bool,

    uid: Uid,

    marker: PhantomData<(Args, Output)>,
}

impl<'a, Args, Set, Ret, Output, H> Register<'a, Set, Ret, H, Args, Output>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
    Ret: Default + 'static,
    H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: ExtractCtx<Set, Error = Error> + 'static,
{
    /// Register the handler with given [`Store`] implementation.
    pub fn with(&mut self, store: impl Store<Set, Output, Ret = Ret, Error = Error> + 'static) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser.register_store(self.uid, handler, store);
            self.register = true;
        }
    }
}

impl<'a, Args, Set, Ret, Output, H> Register<'a, Set, Ret, H, Args, Output>
where
    Output: 'static,
    Set: crate::set::Set,
    Set::Opt: Opt,
    Ret: Default + 'static,
    H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: ExtractCtx<Set, Error = Error> + 'static,
{
    /// Register the handler with default [`ValStore`].
    pub fn with_default(&mut self) {
        if !self.register {
            let handler = self.handler.take().unwrap();

            self.ser
                .register_store(self.uid, handler, ValStore::default());
            self.register = true;
        }
    }
}

impl<'a, Set, Ret, Handler, Args, Output> Drop for Register<'a, Set, Ret, Handler, Args, Output> {
    fn drop(&mut self) {
        if !self.register {
            panic!("Consider call or_default or and_then on Register")
        }
    }
}
