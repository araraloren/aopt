use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::trace;

use crate::astr;
use crate::ctx::wrap_handler;
use crate::ctx::Callbacks;
use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::NullStore;
use crate::ctx::Store;
use crate::opt::Action;
use crate::opt::Assoc;
use crate::opt::Opt;
use crate::opt::RawValParser;
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
/// # use aopt::ext::ServicesExt;
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
///        fn extract(_set: &ASet, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
///            Ok(Self(ctx.args().len()))
///        }
///    }
///    let mut ser = Services::default().with(UsrValService::default());
///    let mut is = InvokeService::new();
///    let mut set = ASet::default();
///    let args = Arc::new(Args::new(["--foo", "bar", "doo"].into_iter()));
///    let ctx = Ctx::default().with_args(args);
///
///    ser.ser_usrval_mut()?.insert(ser::Value::new(42i64));
///    // you can register callback into InvokeService
///    is.entry(0)
///      .on(|_set: &mut ASet, _: &mut ASer| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{0}}");
///            Ok(None)
///        },
///    );
///    is.entry(1)      
///      .on(|_set: &mut ASet, _: &mut ASer, cnt: Count| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{1}}");
///            assert_eq!(cnt.0, 3);
///            Ok(None)
///        },
///    );
///    is.entry(2)
///      .on(|_set: &mut ASet, _: &mut ASer, data: ser::Value<i64>| -> Result<Option<()>, Error> {
///            println!("Calling the handler of {{2}}");
///            assert_eq!(data.as_ref(), &42);
///            Ok(None)
///        },
///    );
///
///    is.invoke(&mut set, &mut ser, &ctx)?;
///    is.invoke(&mut set, &mut ser, &ctx)?;
///    is.invoke(&mut set, &mut ser, &ctx)?;
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
    pub fn set_handler<Args>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Option<()>, Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: Extract<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_handler(handler, NullStore));
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
    pub fn set_handler_with<Args, Output>(
        &mut self,
        uid: Uid,
        handler: impl Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        store: impl Store<Set, Output, Ret = (), Error = Error> + 'static,
    ) -> &mut Self
    where
        Args: Extract<Set, Error = Error> + 'static,
    {
        self.callbacks.insert(uid, wrap_handler(handler, store));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<Set> InvokeService<Set> {
    pub fn entry<Args, Output, H>(&mut self, uid: Uid) -> Entry<'_, Set, H, Args, Output>
    where
        Output: 'static,
        H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
        Args: Extract<Set, Error = Error> + 'static,
    {
        Entry {
            ser: self,
            handler: None,
            register: false,
            uid,
            marker: PhantomData::default(),
        }
    }

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
}

impl<Set> InvokeService<Set>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
{
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
        let mut action = opt.action().clone();

        trace!("Invoke default handler for {{{uid}}}, ctx{{{ctx:?}}}");
        match assoc {
            Assoc::Bool => action.process(uid, set, ser, val, bool::parse(opt, val, ctx).ok()),
            Assoc::Int => action.process(uid, set, ser, val, i64::parse(opt, val, ctx).ok()),
            Assoc::Uint => action.process(uid, set, ser, val, u64::parse(opt, val, ctx).ok()),
            Assoc::Flt => action.process(uid, set, ser, val, f64::parse(opt, val, ctx).ok()),
            Assoc::Str => action.process(uid, set, ser, val, String::parse(opt, val, ctx).ok()),
            Assoc::Noa => action.process(uid, set, ser, val, val.map(|_| true)),
            Assoc::Null => Ok(Some(())),
        }
    }
}

impl<Set> Service for InvokeService<Set> {
    fn service_name() -> Str {
        astr("InvokeService")
    }
}

pub struct Entry<'a, Set, H, Args, Output>
where
    Output: 'static,
    H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: Extract<Set, Error = Error> + 'static,
{
    ser: &'a mut InvokeService<Set>,

    handler: Option<H>,

    register: bool,

    uid: Uid,

    marker: PhantomData<(Args, Output)>,
}

impl<'a, Args, Set, Output, H> Entry<'a, Set, H, Args, Output>
where
    Output: 'static,
    H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: Extract<Set, Error = Error> + 'static,
{
    pub fn on(&mut self, handler: H) -> &mut Self {
        self.handler = Some(handler);
        self
    }

    /// Register the handler with default [`ValStore`].
    pub fn then(&mut self, store: impl Store<Set, Output, Ret = (), Error = Error> + 'static) {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                self.ser.set_handler_with(self.uid, handler, store);
            }
            self.register = true;
        }
    }
}

impl<'a, Set, H, Args, Output> Drop for Entry<'a, Set, H, Args, Output>
where
    Output: 'static,
    H: Handler<Set, Args, Output = Option<Output>, Error = Error> + 'static,
    Args: Extract<Set, Error = Error> + 'static,
{
    fn drop(&mut self) {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                self.ser.set_handler_with(self.uid, handler, Action::Null);
            }
            self.register = true;
        }
    }
}
