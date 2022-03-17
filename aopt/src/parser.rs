mod commit;
mod delay_policy;
mod forward_policy;
mod pre_policy;
mod service;
mod state;
pub(crate) mod testutil;

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use ustr::Ustr;

use crate::arg::Argument;
use crate::err::Result;
use crate::gstr;
use crate::opt::{OptCallback, OptValue};
use crate::proc::{Info, Matcher};
use crate::set::SimpleSet;
use crate::set::{CreateInfo, Set};
use crate::uid::Uid;

pub use commit::CallbackCommit;
pub use delay_policy::DelayPolicy;
pub use forward_policy::ForwardPolicy;
pub use pre_policy::PrePolicy;
pub use service::CallbackStore;
pub use service::SimpleService;
pub use state::ParserState;

pub type DefaultService = SimpleService<SimpleSet>;

/// Default [`Parser`] that using [`ForwardPolicy`].
pub type ForwardParser = Parser<SimpleSet, DefaultService, ForwardPolicy>;

/// Default [`Parser`] that using [`PrePolicy`].
pub type PreParser = Parser<SimpleSet, DefaultService, PrePolicy>;

/// Default [`Parser`] that using [`DelayPolicy`].
pub type DelayParser = Parser<SimpleSet, DefaultService, DelayPolicy>;

#[derive(Debug, Clone)]
pub struct ValueKeeper {
    pub id: Uid,
    pub index: usize,
    pub value: OptValue,
}

/// [`Policy`] doing real parsing work.
///
/// # Example
/// ```ignore
/// #[derive(Debug)]
/// pub struct EmptyPolicy;
///
/// impl<S: Set, SS: Service<S>> Policy<S, SS> for EmptyPolicy {
///     fn parse(
///         &mut self,
///         set: &mut S,
///         service: &mut SS,
///         iter: &mut dyn Iterator<Item = aopt::arg::Argument>,
///     ) -> Result<bool> {
///         // ... parsing logical code
///         Ok(true)
///     }
/// }
/// ```
pub trait Policy<S: Set, SS: Service<S>>: Debug {
    fn parse(
        &mut self,
        set: &mut S,
        service: &mut SS,
        iter: &mut dyn Iterator<Item = Argument>,
    ) -> Result<bool>;
}

/// [`Service`] provide common service using for [`Policy`].
pub trait Service<S: Set> {
    /// Generate M base on [`Argument`] and [`ParserState`].
    fn gen_opt<M: Matcher + Default>(
        &self,
        arg: &Argument,
        style: &ParserState,
        arg_index: u64,
    ) -> Result<Option<M>>;

    /// Generate M base on position information of `NOA` and [`ParserState`].
    fn gen_nonopt<M: Matcher + Default>(
        &self,
        noa: &Ustr,
        total: usize,
        current: usize,
        style: &ParserState,
    ) -> Result<Option<M>>;

    /// Matching the `matcher` with [`Opt`](crate::opt::Opt)s in `set`.
    ///
    /// The `invoke` should be false if caller don't want invoke callback when
    /// [`Opt`](crate::opt::Opt) matched.
    fn matching<M: Matcher + Default>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        invoke: bool,
    ) -> Result<Vec<ValueKeeper>>;

    /// Checking if the `set` data valid.
    fn pre_check(&self, set: &S) -> Result<bool>;

    /// Checking if the `set` data valid.
    fn opt_check(&self, set: &S) -> Result<bool>;

    /// Checking if the `set` data valid.
    fn pos_check(&self, set: &S) -> Result<bool>;

    /// Checking if the `set` data valid.
    fn cmd_check(&self, set: &S) -> Result<bool>;

    /// Checking if the `set` data valid.
    fn post_check(&self, set: &S) -> Result<bool>;

    /// Invoke callback connected with given [`Opt`](crate::opt::Opt).
    fn invoke(
        &mut self,
        uid: Uid,
        set: &mut S,
        noa_idx: usize,
        optvalue: OptValue,
    ) -> Result<Option<OptValue>>;

    /// Return the callback map reference.
    fn get_callback(&self) -> &CallbackStore<S>;

    /// Return the subscriber info vector reference.
    fn get_subscriber_info<I: 'static + Info>(&self) -> &Vec<Box<dyn Info>>;

    /// Return the NOA vector reference.
    fn get_noa(&self) -> &Vec<Ustr>;

    /// Return the callback map mutable reference.
    fn get_callback_mut(&mut self) -> &mut CallbackStore<S>;

    /// Return the subscriber info vector mutable reference.
    fn get_subscriber_info_mut(&mut self) -> &mut Vec<Box<dyn Info>>;

    /// Return the NOA vector mutable reference.
    fn get_noa_mut(&mut self) -> &mut Vec<Ustr>;

    /// Reset the [`Service`].
    fn reset(&mut self);
}

/// Parser manage the [`Set`], [`Service`] and [`Policy`].
///
/// # Example
///
/// ```rust
/// use aopt::err::Result;
/// use aopt::prelude::*;
///
/// fn main() -> Result<()> {
///     #[derive(Debug, Default)]
///     pub struct EmptyPolicy(i64);
///
///     impl<S: Set, SS: Service<S>> Policy<S, SS> for EmptyPolicy {
///         fn parse(
///             &mut self,
///             set: &mut S,
///             service: &mut SS,
///             iter: &mut dyn Iterator<Item = aopt::arg::Argument>,
///         ) -> Result<bool> {
///             println!("In parser policy {} with argument length = {}", self.0, iter.count());
///             Ok(false)
///         }
///     }
///
///     let mut parser1 = Parser::<SimpleSet, DefaultService, EmptyPolicy>::default();
///     let mut parser2 = Parser::<SimpleSet, DefaultService, EmptyPolicy>::new_policy(EmptyPolicy(42));
///
///     getopt!(
///         ["Happy", "Chinese", "new", "year", "!"].into_iter(),
///         parser1,
///         parser2
///     )?;
///     Ok(())
/// }
/// ```
///
/// Using it with macro [`getopt`](crate::getopt),
/// which can process multiple [`Parser`] with same type [`Policy`].
#[derive(Debug)]
pub struct Parser<S, SS, P>
where
    S: Set,
    SS: Service<S>,
    P: Policy<S, SS>,
{
    policy: P,
    service: SS,
    set: S,
}

impl<S, SS, P> Default for Parser<S, SS, P>
where
    S: Set + Default,
    SS: Service<S> + Default,
    P: Policy<S, SS> + Default,
{
    fn default() -> Self {
        Self {
            policy: P::default(),
            service: SS::default(),
            set: S::default(),
        }
    }
}

impl<S, SS, P> Deref for Parser<S, SS, P>
where
    S: Set,
    SS: Service<S>,
    P: Policy<S, SS>,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<S, SS, P> DerefMut for Parser<S, SS, P>
where
    S: Set,
    SS: Service<S>,
    P: Policy<S, SS>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl<S, SS, P> Parser<S, SS, P>
where
    S: Set + Default,
    SS: Service<S> + Default,
    P: Policy<S, SS>,
{
    /// Initialize the [`Parser`] with specify [`Policy`] and the
    /// default value of `S` and `SS`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aopt::err::Result;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<()> {
    ///     #[derive(Debug)]
    ///     pub struct EmptyPolicy;
    ///
    ///     impl<S: Set, SS: Service<S>> Policy<S, SS> for EmptyPolicy {
    ///         fn parse(
    ///             &mut self,
    ///             set: &mut S,
    ///             service: &mut SS,
    ///             iter: &mut dyn Iterator<Item = aopt::arg::Argument>,
    ///         ) -> Result<bool> {
    ///             todo!()
    ///         }
    ///     }
    ///
    ///     dbg!(Parser::<SimpleSet, DefaultService, EmptyPolicy>::new_policy(EmptyPolicy {}));
    ///     Ok(())
    /// }
    /// ```
    pub fn new_policy(policy: P) -> Self {
        Self {
            set: S::default(),
            service: SS::default(),
            policy: policy,
        }
    }
}

impl<S, SS, P> Parser<S, SS, P>
where
    S: Set,
    SS: Service<S>,
    P: Policy<S, SS>,
{
    pub fn new(set: S, service: SS, policy: P) -> Self {
        Self {
            set,
            service,
            policy,
        }
    }

    pub fn get_policy(&self) -> &P {
        &self.policy
    }

    pub fn get_policy_mut(&mut self) -> &mut P {
        &mut self.policy
    }

    pub fn set_policy(&mut self, policy: P) -> &mut Self {
        self.policy = policy;
        self
    }

    pub fn get_service(&self) -> &SS {
        &self.service
    }

    pub fn get_service_mut(&mut self) -> &mut SS {
        &mut self.service
    }

    pub fn set_service(&mut self, service: SS) -> &mut Self {
        self.service = service;
        self
    }

    pub fn get_set(&self) -> &S {
        &self.set
    }

    pub fn get_set_mut(&mut self) -> &mut S {
        &mut self.set
    }

    pub fn set_set(&mut self, set: S) -> &mut Self {
        self.set = set;
        self
    }

    // extern the add_opt function, attach callback to option
    pub fn add_opt_cb(
        &mut self,
        opt_str: &str,
        callback: OptCallback<S>,
    ) -> Result<CallbackCommit<'_, '_, S, SS>> {
        let info = CreateInfo::parse(gstr(opt_str), self.get_prefix())?;

        debug!(%opt_str, "create option has callback");
        Ok(CallbackCommit::new(
            &mut self.set,
            &mut self.service,
            info,
            callback,
        ))
    }

    pub fn add_callback(&mut self, uid: Uid, callback: OptCallback<S>) {
        self.get_service_mut()
            .get_callback_mut()
            .add_callback(uid, callback);
    }

    pub fn parse(&mut self, iter: &mut dyn Iterator<Item = Argument>) -> Result<bool> {
        let service = &mut self.service;
        let policy = &mut self.policy;
        let set = &mut self.set;

        policy.parse(set, service, iter)
    }

    pub fn reset(&mut self) {
        self.service.reset();
        self.set.reset();
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        unsafe impl<S, SS, P> Send for Parser<S, SS, P>
            where S: Set, SS: Service<S>, P: Policy<S, SS> { }

        unsafe impl<S, SS, P> Sync for Parser<S, SS, P>
            where S: Set, SS: Service<S>, P: Policy<S, SS> { }
    }
}

/// DynParser manage the [`Set`], [`Service`] and [`Policy`].
///
/// # Example
///
/// ```no_run
/// use aopt::err::Result;
/// use aopt::prelude::*;
///
/// fn main() -> Result<()> {
///     #[derive(Debug, Default)]
///     pub struct EmptyPolicy;
///
///     impl<S: Set, SS: Service<S>> Policy<S, SS> for EmptyPolicy {
///         fn parse(
///             &mut self,
///             set: &mut S,
///             service: &mut SS,
///             iter: &mut dyn Iterator<Item = aopt::arg::Argument>,
///         ) -> Result<bool> {
///             dbg!(set);
///             Ok(false)
///         }
///     }
///
///     #[derive(Debug, Default)]
///     pub struct ConsumeIterPolicy;
///
///     impl<S: Set, SS: Service<S>> Policy<S, SS> for ConsumeIterPolicy {
///         fn parse(
///             &mut self,
///             set: &mut S,
///             service: &mut SS,
///             iter: &mut dyn Iterator<Item = aopt::arg::Argument>,
///         ) -> Result<bool> {
///             for item in iter {
///                 dbg!(item);
///             }
///             Ok(true)
///         }
///     }
///
///     let mut parser1 = DynParser::<SimpleSet, DefaultService>::new_policy(EmptyPolicy::default());
///
///     let mut parser2 =
///         DynParser::<SimpleSet, DefaultService>::new_policy(ConsumeIterPolicy::default());
///
///     getoptd!(
///         ["Happy", "Chinese", "new", "year", "!"].into_iter(),
///         parser1,
///         parser2
///     )?;
///     Ok(())
/// }
/// ```
///
/// Using it with macro [`getoptd`](crate::getoptd),
/// which can process multiple [`Parser`] with different type [`Policy`].
pub struct DynParser<S, SS>
where
    S: Set,
    SS: Service<S>,
{
    policy: Box<dyn Policy<S, SS>>,
    service: SS,
    set: S,
}

impl<S, SS> Deref for DynParser<S, SS>
where
    S: Set,
    SS: Service<S>,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<S, SS> DerefMut for DynParser<S, SS>
where
    S: Set,
    SS: Service<S>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl<S, SS> DynParser<S, SS>
where
    S: Set + Default,
    SS: Service<S> + Default,
{
    pub fn new_policy<P: Policy<S, SS> + 'static>(policy: P) -> Self {
        Self {
            set: S::default(),
            service: SS::default(),
            policy: Box::new(policy),
        }
    }
}

impl<S, SS> DynParser<S, SS>
where
    S: Set,
    SS: Service<S>,
{
    pub fn new<P: Policy<S, SS> + 'static>(set: S, service: SS, policy: P) -> Self {
        Self {
            set,
            service,
            policy: Box::new(policy),
        }
    }

    pub fn get_policy(&self) -> &dyn Policy<S, SS> {
        self.policy.as_ref()
    }

    pub fn get_policy_mut(&mut self) -> &mut dyn Policy<S, SS> {
        self.policy.as_mut()
    }

    pub fn set_policy<P: Policy<S, SS> + 'static>(&mut self, policy: P) -> &mut Self {
        self.policy = Box::new(policy);
        self
    }

    pub fn get_service(&self) -> &SS {
        &self.service
    }

    pub fn get_service_mut(&mut self) -> &mut SS {
        &mut self.service
    }

    pub fn set_service(&mut self, service: SS) -> &mut Self {
        self.service = service;
        self
    }

    pub fn get_set(&self) -> &S {
        &self.set
    }

    pub fn get_set_mut(&mut self) -> &mut S {
        &mut self.set
    }

    pub fn set_set(&mut self, set: S) -> &mut Self {
        self.set = set;
        self
    }

    // extern the add_opt function, attach callback to option
    pub fn add_opt_cb(
        &mut self,
        opt_str: &str,
        callback: OptCallback<S>,
    ) -> Result<CallbackCommit<'_, '_, S, SS>> {
        let info = CreateInfo::parse(gstr(opt_str), self.get_prefix())?;

        debug!(%opt_str, "create option has callback");
        Ok(CallbackCommit::new(
            &mut self.set,
            &mut self.service,
            info,
            callback,
        ))
    }

    pub fn add_callback(&mut self, uid: Uid, callback: OptCallback<S>) {
        self.get_service_mut()
            .get_callback_mut()
            .add_callback(uid, callback);
    }

    pub fn parse(&mut self, iter: &mut dyn Iterator<Item = Argument>) -> Result<bool> {
        let service = &mut self.service;
        let policy = &mut self.policy;
        let set = &mut self.set;

        policy.parse(set, service, iter)
    }

    pub fn reset(&mut self) {
        self.service.reset();
        self.set.reset();
    }
}

impl<S, SS, P> From<Parser<S, SS, P>> for DynParser<S, SS>
where
    S: Set + Default,
    SS: Service<S> + Default,
    P: Policy<S, SS> + Default + 'static,
{
    fn from(mut parser: Parser<S, SS, P>) -> Self {
        use std::mem::take;

        let set = take(parser.get_set_mut());
        let service = take(parser.get_service_mut());
        let policy = take(parser.get_policy_mut());

        DynParser::new(set, service, policy)
    }
}

impl<'a, S, SS, P> From<&'a mut Parser<S, SS, P>> for DynParser<S, SS>
where
    S: Set + Default,
    SS: Service<S> + Default,
    P: Policy<S, SS> + Default + 'static,
{
    fn from(parser: &'a mut Parser<S, SS, P>) -> Self {
        use std::mem::take;

        let set = take(parser.get_set_mut());
        let service = take(parser.get_service_mut());
        let policy = take(parser.get_policy_mut());

        DynParser::new(set, service, policy)
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        unsafe impl<S, SS> Send for DynParser<S, SS>
            where S: Set, SS: Service<S> { }

        unsafe impl<S, SS> Sync for DynParser<S, SS>
            where S: Set, SS: Service<S> { }
    }
}
