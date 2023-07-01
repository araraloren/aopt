use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ext::APolicyExt;
use crate::parser::HCOptSet;
use crate::parser::OptStyleManager;
use crate::parser::Policy;
use crate::parser::PolicyParser;
use crate::parser::PolicySettings;
use crate::parser::UserStyle;
use crate::set::Set;
use crate::ARef;
use crate::Error;
use crate::Str;

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
