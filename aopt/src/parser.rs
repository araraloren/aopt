pub(crate) mod commit;
pub(crate) mod policy_delay;
pub(crate) mod policy_fwd;
pub(crate) mod policy_pre;
pub(crate) mod process;
pub(crate) mod style;

pub use self::commit::ParserCommit;
pub use self::policy_delay::DelayPolicy;
pub use self::policy_fwd::FwdPolicy;
pub use self::policy_pre::PrePolicy;
pub use self::style::Guess;
pub use self::style::GuessNOACfg;
pub use self::style::GuessOptCfg;
pub use self::style::NOAGuess;
pub use self::style::OptGuess;
pub use self::style::UserStyle;

pub(crate) use self::process::invoke_callback_opt;
pub(crate) use self::process::process_non_opt;
pub(crate) use self::process::process_opt;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ext::ser;
use crate::ext::APolicyExt;
use crate::ext::ServicesExt;
use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Ctor;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::ser::Services;
use crate::set::Commit;
use crate::set::Pre;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetOpt;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

#[derive(Debug, Clone)]
pub struct CtxSaver {
    /// option uid
    pub uid: Uid,

    /// Index of matcher
    pub idx: usize,

    /// invoke context
    pub ctx: Ctx,
}

/// [`Policy`] doing real parsing work.
///
/// # Example
/// ```ignore
///
/// #[derive(Debug)]
/// pub struct EmptyPolicy<S>(PhantomData<S>);
///
/// // An empty policy do nothing.
/// impl<S: Set> Policy for EmptyPolicy<S> {
///     type Ret = bool;
///
///     type Set = S;
///
///     type Error = Error;
///
///     fn parse(&mut self, _: &mut S, _: &mut ASer, _: Arc<Args>) -> Result<Option<bool>, Error> {
///         // ... parsing logical code
///         Ok(Some(true))
///     }
/// }
/// ```
pub trait Policy {
    type Ret;
    type Set: Set;
    type Error: Into<Error>;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        ser: &mut Services,
        args: Arc<Args>,
    ) -> Result<Option<Self::Ret>, Self::Error>;
}

#[derive(Debug)]
pub struct Parser<S, P> {
    optset: S,
    policy: P,
    services: Services,
}

impl<S, P> Default for Parser<S, P>
where
    S: Default,
    P: Default,
{
    fn default() -> Self {
        Self {
            optset: Default::default(),
            policy: Default::default(),
            services: Default::default(),
        }
    }
}

impl<S, P> Deref for Parser<S, P> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.optset
    }
}

impl<S, P> DerefMut for Parser<S, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.optset
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy + APolicyExt<P::Set>,
{
    pub fn new(policy: P) -> Self {
        let set = policy.default_set();
        let services = policy.default_ser();

        Self {
            optset: set,
            policy,
            services,
        }
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy,
{
    pub fn new_with(policy: P, optset: P::Set, services: Services) -> Self {
        Self {
            optset,
            policy,
            services,
        }
    }

    pub fn policy(&self) -> &P {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut P {
        &mut self.policy
    }

    pub fn set_policy(&mut self, policy: P) -> &mut Self {
        self.policy = policy;
        self
    }

    pub fn service(&self) -> &Services {
        &self.services
    }

    pub fn service_mut(&mut self) -> &mut Services {
        &mut self.services
    }

    pub fn set_service(&mut self, services: Services) -> &mut Self {
        self.services = services;
        self
    }

    pub fn optset(&self) -> &P::Set {
        &self.optset
    }

    pub fn optset_mut(&mut self) -> &mut P::Set {
        &mut self.optset
    }

    pub fn set_optset(&mut self, optset: P::Set) -> &mut Self {
        self.optset = optset;
        self
    }

    pub fn set_usr_val<T: 'static>(
        &mut self,
        val: ser::Value<T>,
    ) -> Result<Option<ser::Value<T>>, Error> {
        Ok(self.services.ser_usrval_mut()?.insert(val))
    }

    pub fn val<T: 'static>(&self, uid: Uid) -> Result<&T, Error> {
        self.services.ser_val()?.val::<T>(uid)
    }

    pub fn val_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut T, Error> {
        self.services.ser_val_mut()?.val_mut::<T>(uid)
    }

    pub fn vals<T: 'static>(&self, uid: Uid) -> Result<&Vec<T>, Error> {
        self.services.ser_val()?.vals::<T>(uid)
    }

    pub fn vals_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error> {
        self.services.ser_val_mut()?.vals_mut::<T>(uid)
    }

    pub fn rawval(&self, uid: Uid) -> Result<&RawVal, Error> {
        self.services.ser_rawval()?.val(uid)
    }

    pub fn rawval_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut RawVal, Error> {
        self.services.ser_rawval_mut()?.val_mut(uid)
    }

    pub fn rawvals<T: 'static>(&self, uid: Uid) -> Result<&Vec<RawVal>, Error> {
        self.services.ser_rawval()?.vals(uid)
    }

    pub fn rawvals_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut Vec<RawVal>, Error> {
        self.services.ser_rawval_mut()?.vals_mut(uid)
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy<Error = Error>,
{
    pub fn parse(&mut self, args: Arc<Args>) -> Result<Option<P::Ret>, Error> {
        let optset = &mut self.optset;
        let services = &mut self.services;

        self.policy.parse(optset, services, args)
    }
}

impl<P> Parser<P::Set, P>
where
    P::Set: 'static,
    P: Policy,
    SetOpt<P::Set>: Opt,
    P::Set: Pre + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    pub fn add_opt<Args, Output, H, T: Into<Str>>(
        &mut self,
        opt: T,
    ) -> Result<ParserCommit<'_, P::Set, H, Args, Output>, Error>
    where
        Output: 'static,
        H: Handler<P::Set, Args, Output = Option<Output>, Error = Error> + 'static,
        Args: Extract<P::Set, Error = Error> + 'static,
    {
        let info =
            <<<P::Set as Set>::Ctor as Ctor>::Config as Config>::new(&self.optset, opt.into())?;

        Ok(ParserCommit::new(
            Commit::new(&mut self.optset, info),
            self.services.ser_invoke_mut()?,
        ))
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy,
    P::Set: Set + OptParser,
{
    pub fn val_filter<T: 'static>(&self, opt: &str) -> Result<&T, Error> {
        // let uid = self.optset.
        // self.services.ser_val()?.val::<T>(uid)
        todo!()
    }
}
