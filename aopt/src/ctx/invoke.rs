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
///  let mut ser = ASer::default();
///  let mut is = Invoker::new();
///  let mut set = ASet::default();
///  let orig = Args::from(["--foo", "bar", "doo"]);
///  let args = orig.iter().map(|v|v.as_os_str()).collect();
///  let mut ctx = Ctx::default().with_orig(orig.clone()).with_args(args);
///
///  ser.sve_insert(42i64);
///  // you can register callback into Invoker
///  is.entry(0)
///      .on(
///          |_set: &mut ASet, _: &mut ASer, _: &Ctx| -> Result<Option<()>, Error> {
///              println!("Calling the handler of {{0}}");
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///  is.entry(1)
///      .on(
///          |_set: &mut ASet, _: &mut ASer, ctx: &Ctx| -> Result<Option<()>, Error> {
///              let cnt = ctx.args().len();
///              println!("Calling the handler of {{1}}");
///              assert_eq!(cnt, 3);
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///  is.entry(2)
///      .on(
///          |_set: &mut ASet, ser: &mut ASer, ctx: &Ctx| -> Result<Option<()>, Error> {
///              let data = ser.sve_val::<i64>()?;
///              println!("Calling the handler of {{2}}");
///              assert_eq!(data, &42);
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(0)));
///  is.invoke(&0, &mut set, &mut ser, &mut ctx)?;
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(1)));
///  is.invoke(&1, &mut set, &mut ser, &mut ctx)?;
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(2)));
///  is.invoke(&2, &mut set, &mut ser, &mut ctx)?;
/// #
/// #    Ok(())
/// # }
/// ```
pub struct Invoker<'a, Set, Ser> {
    callbacks: HashMap<Uid, InvokeHandler<'a, Set, Ser, Error>>,
}

impl<'a, Set, Ser> Debug for Invoker<'a, Set, Ser> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Invoker")
            .field("callbacks", &"{ ... }")
            .finish()
    }
}

impl<'a, Set, Ser> Default for Invoker<'a, Set, Ser> {
    fn default() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<'a, Set, Ser> Invoker<'a, Set, Ser> {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<'a, Set, Ser> Invoker<'a, Set, Ser>
where
    Set: crate::set::Set,
{
    pub fn set_raw<H: FnMut(&mut Set, &mut Ser, &mut Ctx) -> Result<bool, Error> + 'a>(
        &mut self,
        uid: Uid,
        handler: H,
    ) -> &mut Self {
        self.callbacks.insert(uid, Box::new(handler));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::parser::Policy) when option set.
    ///
    /// The [`Invoker`] first call the [`invoke`](crate::ctx::Handler::invoke), then
    /// call the [`process`](crate::ctx::Store::process) with the return value.
    /// # Note
    /// ```txt
    /// |   handler: |&mut Set, &mut Ser, ctx: &Ctx| -> Result<Option<Value>, Error>
    /// |   storer: |&mut Set, &mut Ser, Option<&OsStr>, Option<Value>| -> Result<bool, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |&mut Set, &mut Ser, &Ctx| -> Option<Value>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, &mut Set, &mut Ser, &mut Ctx)
    /// |       -> call Store::process(&Set, Option<&OsStr>, Option<Value>)
    /// |           -> Result<bool, Error>
    /// ```
    pub fn set_handler<H, O, S>(&mut self, uid: Uid, handler: H, store: S) -> &mut Self
    where
        O: ErasedTy,
        S: Store<Set, Ser, O, Ret = bool, Error = Error> + 'a,
        H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
    {
        self.set_raw(uid, wrap_handler(handler, store));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }
}

impl<'a, Set, Ser> Invoker<'a, Set, Ser>
where
    SetOpt<Set>: Opt,
    Set: crate::set::Set,
{
    pub fn entry<O, H>(&mut self, uid: Uid) -> HandlerEntry<'a, '_, Self, Set, Ser, H, O>
    where
        O: ErasedTy,
        H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
    {
        HandlerEntry::new(self, uid)
    }

    /// The default handler for all option.
    ///
    /// If there no handler for a option, then default handler will be called.
    /// It will parsing [`OsStr`](using [`RawValParser`](crate::value::RawValParser)) into associated type,
    /// then save the value to [`ValStorer`](crate::value::ValStorer).
    pub fn fallback(set: &mut Set, _: &mut Ser, ctx: &mut Ctx) -> Result<bool, Error> {
        let uid = ctx.uid()?;
        let opt = set.get_mut(uid).unwrap();
        let arg = ctx.arg()?.map(|v| v.as_ref());
        let act = *opt.action();

        trace!("invoke fallback for {}({act}) {{{ctx:?}}}", opt.name());
        opt.accessor_mut().store_all(arg, ctx, &act)
    }
}

/// Handler type using for callback.
pub type InvokeHandler<'a, Set, Ser, Error> =
    Box<dyn FnMut(&mut Set, &mut Ser, &mut Ctx) -> Result<bool, Error> + 'a>;

pub trait HandlerCollection<'a, Set, Ser>
where
    Set: crate::set::Set,
{
    fn register<H: FnMut(&mut Set, &mut Ser, &mut Ctx) -> Result<bool, Error> + 'a>(
        &mut self,
        uid: Uid,
        handler: H,
    );

    fn get_handler(&mut self, uid: &Uid) -> Option<&mut InvokeHandler<'a, Set, Ser, Error>>;

    /// Invoke the handler of given `uid`, will panic if handler not exist.
    fn invoke(
        &mut self,
        uid: &Uid,
        set: &mut Set,
        ser: &mut Ser,
        ctx: &mut Ctx,
    ) -> Result<bool, Error> {
        trace!("invoking callback of {} {:?}", uid, ctx);
        if let Some(callback) = self.get_handler(uid) {
            return (callback)(set, ser, ctx);
        }
        unreachable!(
            "no callback of {}, call `invoke_fb` or `fallback` instead",
            set.opt(*uid)?.name()
        )
    }

    /// Invoke the handler of given `uid` if it exist, otherwise call the [`fallback`](Invoker::fallback).
    fn invoke_fb(
        &mut self,
        uid: &Uid,
        set: &mut Set,
        ser: &mut Ser,
        ctx: &mut Ctx,
    ) -> Result<bool, Error> {
        if let Some(callback) = self.get_handler(uid) {
            trace!("invoking(fb) callback of {} {:?}", uid, ctx);
            (callback)(set, ser, ctx)
        } else {
            trace!("invoking(fb) fallback callback of {} {:?}", uid, ctx);
            Invoker::fallback(set, ser, ctx)
        }
    }
}

impl<'a, Set, Ser> HandlerCollection<'a, Set, Ser> for Invoker<'a, Set, Ser>
where
    Set: crate::set::Set,
{
    fn register<H: FnMut(&mut Set, &mut Ser, &mut Ctx) -> Result<bool, Error> + 'a>(
        &mut self,
        uid: Uid,
        handler: H,
    ) {
        self.set_raw(uid, handler);
    }

    fn get_handler(&mut self, uid: &Uid) -> Option<&mut InvokeHandler<'a, Set, Ser, Error>> {
        self.callbacks.get_mut(uid)
    }
}

pub struct HandlerEntry<'a, 'b, I, Set, Ser, H, O>
where
    O: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    I: HandlerCollection<'a, Set, Ser>,
    H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
{
    ser: &'b mut I,

    uid: Uid,

    marker: PhantomData<(&'a (), O, Set, Ser, H)>,
}

impl<'a, 'b, I, Set, Ser, H, O> HandlerEntry<'a, 'b, I, Set, Ser, H, O>
where
    O: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    I: HandlerCollection<'a, Set, Ser>,
    H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
{
    pub fn new(inv_ser: &'b mut I, uid: Uid) -> Self {
        Self {
            ser: inv_ser,
            uid,
            marker: PhantomData,
        }
    }

    /// Register the handler which will be called when option is set.
    pub fn on(self, handler: H) -> HandlerEntryThen<'a, 'b, I, Set, Ser, H, O> {
        HandlerEntryThen::new(self.ser, self.uid, handler, false)
    }

    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ctx::Invoker::fallback) will be called if
    /// the handler return None.
    pub fn fallback(self, handler: H) -> HandlerEntryThen<'a, 'b, I, Set, Ser, H, O> {
        HandlerEntryThen::new(self.ser, self.uid, handler, true)
    }
}

pub struct HandlerEntryThen<'a, 'b, I, Set, Ser, H, O>
where
    O: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    I: HandlerCollection<'a, Set, Ser>,
    H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
{
    ser: &'b mut I,

    handler: Option<H>,

    register: bool,

    uid: Uid,

    fallback: bool,

    marker: PhantomData<(&'a (), O, Set, Ser)>,
}

impl<'a, 'b, I, Set, Ser, H, O> HandlerEntryThen<'a, 'b, I, Set, Ser, H, O>
where
    O: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    I: HandlerCollection<'a, Set, Ser>,
    H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
{
    pub fn new(ser: &'b mut I, uid: Uid, handler: H, fallback: bool) -> Self {
        Self {
            ser,
            handler: Some(handler),
            register: false,
            uid,
            fallback,
            marker: PhantomData,
        }
    }

    /// Register the handler with given `store`.
    /// The `store` will be used save the return value of option handler.
    pub fn then(mut self, store: impl Store<Set, Ser, O, Ret = bool, Error = Error> + 'a) -> Self {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                if self.fallback {
                    self.ser
                        .register(self.uid, wrap_handler_fallback(handler, store));
                } else {
                    self.ser.register(self.uid, wrap_handler(handler, store));
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
                    self.ser
                        .register(self.uid, wrap_handler_fallback_action(handler));
                } else {
                    self.ser.register(self.uid, wrap_handler_action(handler));
                }
            }
            self.register = true;
        }
        self.uid
    }
}

impl<'a, 'b, I, Set, Ser, H, O> Drop for HandlerEntryThen<'a, 'b, I, Set, Ser, H, O>
where
    O: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    I: HandlerCollection<'a, Set, Ser>,
    H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
{
    fn drop(&mut self) {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                if self.fallback {
                    self.ser
                        .register(self.uid, wrap_handler_fallback_action(handler));
                } else {
                    self.ser.register(self.uid, wrap_handler_action(handler));
                }
            }
            self.register = true;
        }
    }
}
