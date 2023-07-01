pub(crate) mod checker;
pub(crate) mod commit;
pub(crate) mod failure;
pub(crate) mod optset;
pub(crate) mod parser;
pub(crate) mod policy_delay;
pub(crate) mod policy_fwd;
pub(crate) mod policy_pre;
pub(crate) mod process;
pub(crate) mod returnval;
pub(crate) mod style;

pub use self::checker::DefaultSetChecker;
pub use self::commit::ParserCommit;
pub use self::commit::ParserCommitWithValue;
pub use self::failure::FailManager;
pub use self::optset::HCOptSet;
pub use self::parser::Parser;
pub use self::policy_delay::DelayPolicy;
pub use self::policy_fwd::FwdPolicy;
pub use self::policy_pre::PrePolicy;
pub use self::returnval::ReturnVal;
pub use self::style::Guess;
pub use self::style::GuessNOACfg;
pub use self::style::GuessOptCfg;
pub use self::style::NOAGuess;
pub use self::style::OptGuess;
pub use self::style::OptStyleManager;
pub use self::style::UserStyle;

pub(crate) use self::process::invoke_callback_opt;
pub(crate) use self::process::process_callback_ret;
pub(crate) use self::process::process_non_opt;
pub(crate) use self::process::process_opt;
pub(crate) use self::process::ProcessCtx;

use std::fmt::Debug;

use crate::args::Args;
use crate::ctx::InnerCtx;
use crate::ARef;
use crate::Error;
use crate::Str;
use crate::Uid;

#[derive(Debug, Clone)]
pub struct CtxSaver {
    /// option uid
    pub uid: Uid,

    /// Index of matcher
    pub idx: usize,

    /// invoke context
    pub ctx: InnerCtx,
}

/// [`Policy`] doing real parsing work.
///
/// # Example
/// ```ignore
///
/// #[derive(Debug)]
/// pub struct EmptyPolicy<Set, Ser>(PhantomData<(Set, Ser)>);
///
/// // An empty policy do nothing.
/// impl<S: Set, Ser> Policy for EmptyPolicy<S, Ser> {
///     type Ret = bool;
///
///     type Set = S;
///
///     type Inv<'a> = Invoker<'a, S, Ser>;
///
///     type Ser = Ser;
///
///     type Error = Error;
///
///     fn parse<'a>(
///         &mut self,
///         _: &mut Self::Set,
///         _: &mut Self::Inv<'a>,
///         _: &mut Self::Ser,
///         _: ARef<Args>,
///    ) -> Result<bool, Error> {
///         // ... parsing logical code
///        Ok(true)
///     }
/// }
/// ```
pub trait Policy {
    type Ret;
    type Set;
    type Inv<'a>;
    type Ser;
    type Error: Into<Error>;

    fn parse<'a>(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv<'a>,
        ser: &mut Self::Ser,
        args: ARef<Args>,
    ) -> Result<Self::Ret, Self::Error>;
}

pub trait PolicySettings {
    fn style_manager(&self) -> &OptStyleManager;

    fn style_manager_mut(&mut self) -> &mut OptStyleManager;

    fn strict(&self) -> bool;

    fn styles(&self) -> &[UserStyle];

    fn no_delay(&self) -> Option<&[Str]>;

    fn set_strict(&mut self, strict: bool) -> &mut Self;

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self;

    fn set_no_delay(&mut self, name: impl Into<Str>) -> &mut Self;
}

pub trait PolicyParser<P>
where
    P: Policy,
{
    type Error: Into<Error>;

    fn parse_env(&mut self) -> Result<P::Ret, Self::Error>
    where
        P: Default,
    {
        self.parse(ARef::new(Args::from_env()))
    }

    fn parse(&mut self, args: ARef<Args>) -> Result<P::Ret, Self::Error>
    where
        P: Default;

    fn parse_env_args(&mut self) -> Result<P::Ret, Self::Error>
    where
        P: Default,
    {
        let mut policy = P::default();
        let args = ARef::new(Args::from_env());
        self.parse_policy(args, &mut policy)
    }

    fn parse_args(&mut self, args: ARef<Args>) -> Result<P::Ret, Self::Error>
    where
        P: Default,
    {
        let mut policy = P::default();
        self.parse_policy(args, &mut policy)
    }

    fn parse_env_policy(&mut self, policy: &mut P) -> Result<P::Ret, Self::Error> {
        let args = ARef::new(Args::from_env());
        self.parse_policy(args, policy)
    }

    fn parse_policy(&mut self, args: ARef<Args>, policy: &mut P) -> Result<P::Ret, Self::Error>;
}
