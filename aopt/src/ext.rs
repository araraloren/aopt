use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Invoker;
use crate::opt::AOpt;
use crate::opt::BuiltInCtor;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::OptParser;
use crate::opt::StrParser;
use crate::parser::DefaultSetChecker;
use crate::parser::DelayPolicy;
use crate::parser::FwdPolicy;
use crate::parser::OptStyleManager;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::PolicyParser;
use crate::parser::PolicySettings;
use crate::parser::PrePolicy;
use crate::parser::UserStyle;
use crate::ser::AppServices;
use crate::set::Ctor;
use crate::set::OptSet;
use crate::set::OptValidator;
use crate::set::PrefixOptValidator;
use crate::set::SetChecker;
use crate::Error;
use crate::Str;

pub mod ctx;
pub mod ser;

/// Generate default value for type.
pub trait ADefaultVal {
    fn a_default_val() -> Self;
}

pub trait APolicyExt<P: Policy> {
    fn default_ser(&self) -> P::Ser;

    fn default_set(&self) -> P::Set;

    fn default_inv<'a>(&self) -> P::Inv<'a>;
}

impl<Set, Ser, Chk> APolicyExt<FwdPolicy<Set, Ser, Chk>> for FwdPolicy<Set, Ser, Chk>
where
    Chk: SetChecker<Set>,
    Ser: Default + 'static,
    Set: crate::set::Set + OptParser + OptValidator + ADefaultVal + 'static,
{
    fn default_set(&self) -> Set {
        Set::a_default_val()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <FwdPolicy<Set, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<Set, Ser>::default()
    }
}

impl<Set, Ser, Chk> APolicyExt<PrePolicy<Set, Ser, Chk>> for PrePolicy<Set, Ser, Chk>
where
    Ser: Default + 'static,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser + OptValidator + ADefaultVal + 'static,
{
    fn default_set(&self) -> Set {
        Set::a_default_val()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <PrePolicy<Set, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<Set, Ser>::default()
    }
}

impl<Set, Ser, Chk> APolicyExt<DelayPolicy<Set, Ser, Chk>> for DelayPolicy<Set, Ser, Chk>
where
    Ser: Default + 'static,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser + OptValidator + ADefaultVal + 'static,
{
    fn default_set(&self) -> Set {
        Set::a_default_val()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <DelayPolicy<Set, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<Set, Ser>::default()
    }
}

pub type ACreator = Creator<AOpt, OptConfig, Error>;

pub type ASet = OptSet<StrParser, ACreator, PrefixOptValidator>;

pub type ASer = AppServices;

pub type AInvoker<'a> = Invoker<'a, ASet, ASer>;

pub type AFwdPolicy = FwdPolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type APrePolicy = PrePolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type ADelayPolicy = DelayPolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type AParser<'a> = Parser<ASet, AInvoker<'a>, ASer>;

pub type AFwdParser<'a> = APolicyParser<'a, AFwdPolicy>;

pub type APreParser<'a> = APolicyParser<'a, APrePolicy>;

pub type ADelayParser<'a> = APolicyParser<'a, ADelayPolicy>;

pub struct APolicyParser<'a, P: Policy>(PolicyParser<'a, P>);

impl<'a, P> Default for APolicyParser<'a, P>
where
    P: Policy + ADefaultVal,
    P::Set: ADefaultVal,
    P::Inv<'a>: ADefaultVal,
    P::Ser: ADefaultVal,
{
    fn default() -> Self {
        Self::a_default_val()
    }
}

impl<'a, P: Policy> std::fmt::Debug for APolicyParser<'a, P>
where
    P::Set: Debug,
    P::Inv<'a>: Debug,
    P::Ser: Debug,
    P: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("APolicyParser").field(&self.0).finish()
    }
}

impl<'a, P: Policy> Deref for APolicyParser<'a, P> {
    type Target = PolicyParser<'a, P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, P: Policy> DerefMut for APolicyParser<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, P> PolicySettings for APolicyParser<'a, P>
where
    P: Policy + PolicySettings,
{
    fn style_manager(&self) -> &OptStyleManager {
        PolicyParser::style_manager(self)
    }

    fn style_manager_mut(&mut self) -> &mut OptStyleManager {
        PolicyParser::style_manager_mut(self)
    }

    fn strict(&self) -> bool {
        PolicyParser::strict(self)
    }

    fn styles(&self) -> &[UserStyle] {
        PolicyParser::styles(self)
    }

    fn no_delay(&self) -> Option<&[Str]> {
        PolicyParser::no_delay(self)
    }

    fn set_strict(&mut self, strict: bool) -> &mut Self {
        PolicyParser::set_strict(self, strict);
        self
    }

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        PolicyParser::set_styles(self, styles);
        self
    }

    fn set_no_delay(&mut self, name: impl Into<Str>) -> &mut Self {
        PolicyParser::set_no_delay(self, name);
        self
    }
}

/// Return an [`ASet`](crate::ext::ASet) with follow creators:
///
/// * [`Fallback`](BuiltInCtor::Fallback)
/// * [`Int`](BuiltInCtor::Int)
/// * [`Bool`](BuiltInCtor::Bool)
/// * [`Flt`](BuiltInCtor::Flt)
/// * [`Str`](BuiltInCtor::Str)
/// * [`Uint`](BuiltInCtor::Uint)
/// * [`Cmd`](BuiltInCtor::Cmd)
/// * [`Pos`](BuiltInCtor::Pos)
/// * [`Main`](BuiltInCtor::Main)
/// * [`Any`](BuiltInCtor::Any)
/// * [`Raw`](BuiltInCtor::Raw)
#[macro_export]
macro_rules! aset {
    () => {
        $crate::aset!(
            fallback,
            int,
            bool,
            flt,
            str,
            uint,
            cmd,
            pos,
            main,
            any,
            raw
        )
    };
    ($($creator:ident),+) => {
        {
            let mut set = $crate::ext::ASet::default();
            $(
                set = set.with_creator(
                    $crate::opt::Creator::from(
                        $crate::opt::BuiltInCtor::from_name(
                            &stringify!($creator)
                        )));
            )+
            set
        }
    };
    ($parser:ident, $ctor:ident, $validator:ident { }) => {
        $crate::aset!(
            $parser, $ctor, $validator {
            fallback,
            int,
            bool,
            flt,
            str,
            uint,
            cmd,
            pos,
            main,
            any,
            raw
        }
        )
    };
    ($parser:ident, $ctor:ident, $validator:ident { $($creator:ident),+ }) => {
        {
            let mut set = <$crate::set::OptSet<$parser, $ctor, $validator>>::default();
            $(
                set = set.with_creator(
                    <$ctor>::from(
                        $crate::opt::BuiltInCtor::from_name(
                            &stringify!($creator)
                        )));
            )+
            set
        }
    };
}

impl<P, C, V> ADefaultVal for OptSet<P, C, V>
where
    C: Ctor + From<BuiltInCtor>,
    C::Opt: crate::opt::Opt,
    P: OptParser + Default,
    V: OptValidator + Default,
{
    fn a_default_val() -> Self {
        crate::aset!(P, C, V {})
    }
}

impl ADefaultVal for ASer {
    fn a_default_val() -> Self {
        ASer::default()
    }
}

impl<'a, Set, Ser> ADefaultVal for Invoker<'a, Set, Ser> {
    fn a_default_val() -> Self {
        Invoker::default()
    }
}

impl<Set, Ser, Chk> ADefaultVal for FwdPolicy<Set, Ser, Chk>
where
    Chk: Default,
{
    fn a_default_val() -> Self {
        FwdPolicy::default()
    }
}

impl<Set, Ser, Chk> ADefaultVal for DelayPolicy<Set, Ser, Chk>
where
    Chk: Default,
{
    fn a_default_val() -> Self {
        DelayPolicy::default()
    }
}

impl<Set, Ser, Chk> ADefaultVal for PrePolicy<Set, Ser, Chk>
where
    Chk: Default,
{
    fn a_default_val() -> Self {
        PrePolicy::default()
    }
}

impl<'a, Set, Inv, Ser> ADefaultVal for Parser<Set, Inv, Ser>
where
    Set: ADefaultVal,
    Inv: ADefaultVal,
    Ser: ADefaultVal,
{
    fn a_default_val() -> Self {
        Parser::new(
            Set::a_default_val(),
            Inv::a_default_val(),
            Ser::a_default_val(),
        )
    }
}

impl<'a, P> ADefaultVal for PolicyParser<'a, P>
where
    P: Policy + ADefaultVal,
    P::Set: ADefaultVal,
    P::Inv<'a>: ADefaultVal,
    P::Ser: ADefaultVal,
{
    fn a_default_val() -> Self {
        let set = <P::Set>::a_default_val();
        let ser = <P::Ser>::a_default_val();
        let inv = <P::Inv<'_>>::a_default_val();

        PolicyParser::new_with(P::a_default_val(), set, inv, ser)
    }
}

impl<'a, P> ADefaultVal for APolicyParser<'a, P>
where
    P: Policy + ADefaultVal,
    P::Set: ADefaultVal,
    P::Inv<'a>: ADefaultVal,
    P::Ser: ADefaultVal,
{
    fn a_default_val() -> Self {
        let set = <P::Set>::a_default_val();
        let ser = <P::Ser>::a_default_val();
        let inv = <P::Inv<'_>>::a_default_val();

        Self(PolicyParser::new_with(P::a_default_val(), set, inv, ser))
    }
}
