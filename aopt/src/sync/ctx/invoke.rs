use std::fmt::Debug;
use std::marker::PhantomData;

use crate::ctx::wrap_handler;
use crate::ctx::wrap_handler_action;
use crate::ctx::wrap_handler_fallback;
use crate::ctx::wrap_handler_fallback_action;
use crate::ctx::Ctx;
use crate::ctx::Store;
use crate::map::ErasedTy;
use crate::opt::Opt;
use crate::set::Set;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::trace;
use crate::Error;
use crate::HashMap;
use crate::Uid;

/// Keep the variable length arguments handler in [`HashMap`] with key [`Uid`].
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
///  let mut is = AInvoker::default();
///  let mut set = AHCSet::default();
///  let orig = Args::from(["--foo", "bar", "doo"]);
///  let args = orig.iter().map(|v|v.as_os_str()).collect();
///  let mut ctx = Ctx::default().with_orig(orig.clone()).with_args(args);
///
///  set.set_app_data(42i64);
///  // you can register callback into Invoker
///  is.entry(0)
///      .on(
///          |_, _| -> Result<Option<()>, Error> {
///              println!("Calling the handler of {{0}}");
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///  is.entry(1)
///      .on(
///          |_, ctx| -> Result<Option<()>, Error> {
///              let cnt = ctx.args().len();
///              println!("Calling the handler of {{1}}");
///              assert_eq!(cnt, 3);
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///  is.entry(2)
///      .on(
///          |set, _| -> Result<Option<()>, Error> {
///              let data = set.app_data::<i64>()?;
///              println!("Calling the handler of {{2}}");
///              assert_eq!(data, &42);
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(0)));
///  is.invoke(&0, &mut set, &mut ctx)?;
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(1)));
///  is.invoke(&1, &mut set, &mut ctx)?;
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(2)));
///  is.invoke(&2, &mut set, &mut ctx)?;
/// #
/// #    Ok(())
/// # }
/// ```
pub struct Invoker<'a, S> {
    callbacks: HashMap<Uid, InvokeHandler<'a, S, Error>>,
}

impl<'a, S> Debug for Invoker<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Invoker")
            .field("callbacks", &"{ ... }")
            .finish()
    }
}

impl<'a, S> Default for Invoker<'a, S> {
    fn default() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<'a, S> Invoker<'a, S> {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<'a, S: Set> Invoker<'a, S> {
    pub fn set_raw<H: FnMut(&mut S, &mut Ctx) -> Result<bool, Error> + Send + Sync + 'a>(
        &mut self,
        uid: Uid,
        handler: H,
    ) -> &mut Self {
        self.callbacks.insert(uid, Box::new(handler));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::parser::Policy) when option set.
    ///
    /// The [`Invoker`] first call the handler, then
    /// call the [`process`](crate::ctx::Store::process) with the return value.
    /// # Note
    /// ```txt
    /// |   handler: |&mut Set, ctx: &mut Ctx| -> Result<Option<Value>, Error>
    /// |   storer: |&mut Set, Option<&OsStr>, Option<Value>| -> Result<bool, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |&mut Set, &mut Ctx| -> Option<Value>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, &mut Set, &mut Ctx)
    /// |       -> call Store::process(&Set, Option<&OsStr>, Option<Value>)
    /// |           -> Result<bool, Error>
    /// ```
    pub fn set_handler<O, H, T>(&mut self, uid: Uid, handler: H, store: T) -> &mut Self
    where
        O: ErasedTy,
        T: Store<S, O, Ret = bool, Error = Error> + Send + Sync + 'a,
        H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
    {
        self.set_raw(uid, wrap_handler(handler, store));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<'a, S> Invoker<'a, S>
where
    SetOpt<S>: Opt,
    S: Set,
{
    pub fn entry<O, H>(&mut self, uid: Uid) -> HandlerEntry<'a, '_, Self, S, H, O>
    where
        O: ErasedTy,
        H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
    {
        HandlerEntry::new(self, uid)
    }

    /// The default handler for all option.
    ///
    /// If there no handler for a option, then default handler will be called.
    /// It will parsing [`OsStr`](std::ffi::OsStr)(using [`RawValParser`](crate::value::RawValParser)) into associated type,
    /// then save the value to [`ValStorer`](crate::value::ValStorer).
    pub fn fallback(set: &mut S, ctx: &mut Ctx) -> Result<bool, Error> {
        let uid = ctx.uid()?;
        let opt = set.get_mut(uid).unwrap();
        let arg = ctx.arg()?.map(|v| v.as_ref());
        let act = *opt.action();

        trace!("in fallback, call for {}({act}) {{{ctx:?}}}", opt.name());
        opt.accessor_mut().store_all(arg, ctx, &act)
    }
}

/// Handler type using for callback.
pub type InvokeHandler<'a, S, Error> =
    Box<dyn FnMut(&mut S, &mut Ctx) -> Result<bool, Error> + Send + Sync + 'a>;

pub trait HandlerCollection<'a, S: Set> {
    fn register_handler<H: FnMut(&mut S, &mut Ctx) -> Result<bool, Error> + Send + Sync + 'a>(
        &mut self,
        uid: Uid,
        handler: H,
    );

    fn get_handler(&mut self, uid: &Uid) -> Option<&mut InvokeHandler<'a, S, Error>>;

    /// Invoke the handler saved in [`Invoker`], it will panic if the handler not exist.
    fn invoke(&mut self, uid: &Uid, set: &mut S, ctx: &mut Ctx) -> Result<bool, Error> {
        trace!("invoking callback of {} {:?}", uid, ctx);
        if let Some(callback) = self.get_handler(uid) {
            return (callback)(set, ctx);
        }
        unreachable!(
            "no callback of {}, call `invoke_fb` or `fallback` instead",
            set.opt(*uid)?.name()
        )
    }

    /// Invoke the handler of given `uid` if it exist, otherwise call the [`fallback`](Invoker::fallback).
    fn invoke_fb(&mut self, uid: &Uid, set: &mut S, ctx: &mut Ctx) -> Result<bool, Error> {
        if let Some(callback) = self.get_handler(uid) {
            trace!("invoking(fb) callback of {} {:?}", uid, ctx);
            (callback)(set, ctx)
        } else {
            trace!("invoking(fb) fallback callback of {} {:?}", uid, ctx);
            Invoker::fallback(set, ctx)
        }
    }
}

impl<'a, S: Set> HandlerCollection<'a, S> for Invoker<'a, S> {
    fn register_handler<H: FnMut(&mut S, &mut Ctx) -> Result<bool, Error> + Send + Sync + 'a>(
        &mut self,
        uid: Uid,
        handler: H,
    ) {
        self.set_raw(uid, handler);
    }

    fn get_handler(&mut self, uid: &Uid) -> Option<&mut InvokeHandler<'a, S, Error>> {
        self.callbacks.get_mut(uid)
    }
}

pub struct HandlerEntry<'a, 'b, I, S, H, O>
where
    O: ErasedTy,
    S: Set,
    SetOpt<S>: Opt,
    I: HandlerCollection<'a, S>,
    H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
{
    invoker: &'b mut I,

    uid: Uid,

    marker: PhantomData<(&'a (), O, S, H)>,
}

impl<'a, 'b, I, S, H, O> HandlerEntry<'a, 'b, I, S, H, O>
where
    O: ErasedTy,
    S: Set,
    SetOpt<S>: Opt,
    I: HandlerCollection<'a, S>,
    H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
{
    pub fn new(invoker: &'b mut I, uid: Uid) -> Self {
        Self {
            invoker,
            uid,
            marker: PhantomData,
        }
    }

    /// Register the handler which will be called when option is set.
    pub fn on(self, handler: H) -> HandlerEntryThen<'a, 'b, I, S, H, O> {
        HandlerEntryThen::new(self.invoker, self.uid, handler, false)
    }

    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ctx::Invoker::fallback) will be called if
    /// the handler return None.
    pub fn fallback(self, handler: H) -> HandlerEntryThen<'a, 'b, I, S, H, O> {
        HandlerEntryThen::new(self.invoker, self.uid, handler, true)
    }
}

pub struct HandlerEntryThen<'a, 'b, I, S, H, O>
where
    O: ErasedTy,
    S: Set,
    SetOpt<S>: Opt,
    I: HandlerCollection<'a, S>,
    H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
{
    invoker: &'b mut I,

    handler: Option<H>,

    register: bool,

    uid: Uid,

    fallback: bool,

    marker: PhantomData<(&'a (), O, S)>,
}

impl<'a, 'b, I, S, H, O> HandlerEntryThen<'a, 'b, I, S, H, O>
where
    O: ErasedTy,
    S: Set,
    SetOpt<S>: Opt,
    I: HandlerCollection<'a, S>,
    H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
{
    pub fn new(invoker: &'b mut I, uid: Uid, handler: H, fallback: bool) -> Self {
        Self {
            invoker,
            handler: Some(handler),
            register: false,
            uid,
            fallback,
            marker: PhantomData,
        }
    }

    /// Register the handler with given `store`.
    /// The `store` will be used save the return value of option handler.
    pub fn then(
        mut self,
        store: impl Store<S, O, Ret = bool, Error = Error> + Send + Sync + 'a,
    ) -> Self {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                if self.fallback {
                    self.invoker
                        .register_handler(self.uid, wrap_handler_fallback(handler, store));
                } else {
                    self.invoker
                        .register_handler(self.uid, wrap_handler(handler, store));
                }
            }
            self.register = true;
        }
        self
    }

    pub fn submit(mut self) -> Uid {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                if self.fallback {
                    self.invoker
                        .register_handler(self.uid, wrap_handler_fallback_action(handler));
                } else {
                    self.invoker
                        .register_handler(self.uid, wrap_handler_action(handler));
                }
            }
            self.register = true;
        }
        self.uid
    }
}

impl<'a, 'b, I, S, H, O> Drop for HandlerEntryThen<'a, 'b, I, S, H, O>
where
    O: ErasedTy,
    S: Set,
    SetOpt<S>: Opt,
    I: HandlerCollection<'a, S>,
    H: FnMut(&mut S, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
{
    fn drop(&mut self) {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                if self.fallback {
                    self.invoker
                        .register_handler(self.uid, wrap_handler_fallback_action(handler));
                } else {
                    self.invoker
                        .register_handler(self.uid, wrap_handler_action(handler));
                }
            }
            self.register = true;
        }
    }
}
