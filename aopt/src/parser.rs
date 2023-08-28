pub(crate) mod checker;
pub(crate) mod commit;
pub(crate) mod failure;
pub(crate) mod optset;
pub(crate) mod policy_delay;
pub(crate) mod policy_fwd;
pub(crate) mod policy_pre;
pub(crate) mod returnval;
pub(crate) mod style;

pub use self::checker::DefaultSetChecker;
pub use self::commit::ParserCommit;
pub use self::commit::ParserCommitWithValue;
pub use self::failure::FailManager;
pub use self::optset::HCOptSet;
pub use self::policy_delay::DelayPolicy;
pub use self::policy_fwd::FwdPolicy;
pub use self::policy_pre::PrePolicy;
pub use self::returnval::ReturnVal;
pub use self::style::OptStyleManager;
pub use self::style::UserStyle;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ctx::InnerCtx;
use crate::ext::APolicyExt;
use crate::set::Set;
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

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv<'_>,
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
        P: Default,
    {
        let mut policy = P::default();
        self.parse_policy(args, &mut policy)
    }

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

/// Parser manage the components are using in [`parse`](Policy::parse) of [`Policy`].
///
/// # Example
///
/// ```rust
/// # use aopt::getopt;
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut parser1 = Parser::new_policy(AFwdPolicy::default());
///
/// parser1.add_opt("Where=c")?;
/// parser1.add_opt("question=m")?.on(question)?;
///
/// let mut parser2 = Parser::new_policy(AFwdPolicy::default());
///
/// parser2.add_opt("Who=c")?;
/// parser2.add_opt("question=m")?.on(question)?;
///
/// fn question(_: &mut ASet, _: &mut ASer, args: ctx::Args) -> Result<Option<()>, Error> {
///     // Output: The question is: Where are you from ?
///     println!(
///         "The question is: {}",
///         args.iter().skip(1)
///             .map(|v| v.get_str().unwrap().to_owned())
///             .collect::<Vec<String>>()
///             .join(" ")
///     );
///     Ok(Some(()))
/// }
///
/// let ret = getopt!(
///     Args::from_array(["app", "Where", "are", "you", "from", "?"]),
///     &mut parser1,
///     &mut parser2
/// )?;
/// let parser = ret.parser;
///
/// assert_eq!(
///     parser[0].name(),
///     "Where",
///     "Parser with `Where` cmd matched"
/// );
/// #
/// # Ok(())
/// # }
/// ```
///
/// Using it with macro [`getopt`](crate::getopt),
/// which can process multiple [`Parser`] with same type [`Policy`].
#[derive(Debug, Default)]
pub struct Parser<'a, P: Policy> {
    policy: P,
    optset: HCOptSet<P::Set, P::Inv<'a>, P::Ser>,
}

impl<'a, P: Policy> Deref for Parser<'a, P> {
    type Target = HCOptSet<P::Set, P::Inv<'a>, P::Ser>;

    fn deref(&self) -> &Self::Target {
        &self.optset
    }
}

impl<'a, P: Policy> DerefMut for Parser<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.optset
    }
}

impl<'a, P> Parser<'a, P>
where
    P: Policy + APolicyExt<P>,
{
    pub fn new_policy(policy: P) -> Self {
        let optset = policy.default_set();
        let valser = policy.default_ser();
        let invoker = policy.default_inv();

        Self {
            policy,
            optset: HCOptSet::new(optset, invoker, valser),
        }
    }
}

impl<'a, P: Policy> Parser<'a, P> {
    pub fn new(
        policy: P,
        optset: HCOptSet<<P as Policy>::Set, <P as Policy>::Inv<'a>, <P as Policy>::Ser>,
    ) -> Self {
        Self { optset, policy }
    }

    pub fn new_with(policy: P, optset: P::Set, invoker: P::Inv<'a>, valser: P::Ser) -> Self {
        Self {
            policy,
            optset: HCOptSet::new(optset, invoker, valser),
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

    pub fn optset(&self) -> &HCOptSet<P::Set, P::Inv<'a>, P::Ser> {
        &self.optset
    }

    pub fn optset_mut(&mut self) -> &mut HCOptSet<P::Set, P::Inv<'a>, P::Ser> {
        &mut self.optset
    }

    pub fn set_optset(&mut self, optset: HCOptSet<P::Set, P::Inv<'a>, P::Ser>) -> &mut Self {
        self.optset = optset;
        self
    }
}

impl<'a, P> Parser<'a, P>
where
    P::Set: Set,
    P: Policy,
{
    /// Reset the option set.
    pub fn reset(&mut self) -> Result<&mut Self, Error> {
        self.optset.reset()?;
        Ok(self)
    }

    /// Call the [`init`](crate::opt::Opt::init) of [`Opt`](crate::opt::Opt) initialize the option value.
    pub fn init(&mut self) -> Result<(), Error> {
        self.optset.init()
    }
}

impl<'a, P> PolicySettings for Parser<'a, P>
where
    P: Policy + PolicySettings,
{
    fn style_manager(&self) -> &OptStyleManager {
        self.policy().style_manager()
    }

    fn style_manager_mut(&mut self) -> &mut OptStyleManager {
        self.policy_mut().style_manager_mut()
    }

    fn strict(&self) -> bool {
        self.policy().strict()
    }

    fn styles(&self) -> &[UserStyle] {
        self.policy.styles()
    }

    fn no_delay(&self) -> Option<&[Str]> {
        self.policy().no_delay()
    }

    fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.policy_mut().set_strict(strict);
        self
    }

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.policy_mut().set_styles(styles);
        self
    }

    fn set_no_delay(&mut self, name: impl Into<Str>) -> &mut Self {
        self.policy_mut().set_no_delay(name);
        self
    }
}

impl<'a, P> Parser<'a, P>
where
    P: Policy + PolicySettings,
{
    /// Enable [`CombinedOption`](UserStyle::CombinedOption) option set style.
    /// This can support option style like `-abc` which set `-a`, `-b` and `-c` both.
    pub fn enable_combined(&mut self) -> &mut Self {
        self.style_manager_mut().push(UserStyle::CombinedOption);
        self
    }

    /// Enable [`EmbeddedValuePlus`](UserStyle::EmbeddedValuePlus) option set style.
    /// This can support option style like `--opt42` which set `--opt` value to 42.
    /// In default the [`EmbeddedValue`](UserStyle::EmbeddedValue) style only support
    /// one letter option such as `-i`.
    pub fn enable_embedded_plus(&mut self) -> &mut Self {
        self.style_manager_mut().push(UserStyle::EmbeddedValuePlus);
        self
    }

    /// Enable [`Flag`](UserStyle::Flag) option set style.
    /// This can support option style like `--opt42` which set `--opt` value to 42.
    /// In default the [`Flag`](UserStyle::Flag) style only support
    /// one letter option such as `-i`.
    pub fn enable_flag(&mut self) -> &mut Self {
        self.style_manager_mut().push(UserStyle::Flag);
        self
    }
}

impl<'a, P: Policy> PolicyParser<P> for Parser<'a, P>
where
    P::Set: crate::set::Set,
{
    type Error = Error;

    fn parse_policy(
        &mut self,
        args: ARef<Args>,
        policy: &mut P,
    ) -> Result<<P as Policy>::Ret, Self::Error> {
        PolicyParser::<P>::parse_policy(&mut self.optset, args, policy)
    }
}
