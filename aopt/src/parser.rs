pub(crate) mod policy_delay;
pub(crate) mod policy_fwd;
pub(crate) mod policy_pre;
pub(crate) mod process;
pub(crate) mod style;

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

use crate::args::Args;
use crate::ctx::Ctx;
use crate::ext::APolicyExt;
use crate::ser::Services;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::Uid;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

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

impl<S, P> Parser<S, P>
where
    S: Set,
    P: Policy + APolicyExt<S>,
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

impl<S, P> Parser<S, P> {
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

    pub fn optset(&self) -> &S {
        &self.optset
    }

    pub fn optset_mut(&mut self) -> &mut S {
        &mut self.optset
    }

    pub fn set_optset(&mut self, optset: S) -> &mut Self {
        self.optset = optset;
        self
    }
}

impl<S, P> Parser<S, P>
where
    S: Set,
    P: Policy,
{
    pub fn new_with(policy: P, optset: S, services: Services) -> Self {
        Self {
            optset,
            policy,
            services,
        }
    }
}
