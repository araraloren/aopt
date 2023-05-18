use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Invoker;
use crate::opt::AOpt;
use crate::opt::Creator;
use crate::opt::OptConfig;
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
use crate::set::SetChecker;
use crate::ser::AppServices;
use crate::set::OptSet;
use crate::set::PrefixOptValidator;
use crate::Error;
use crate::Str;

pub mod ctx;
pub mod ser;

/// Generate default value for type.
pub trait ANewDefault {
    fn new_default() -> Self;
}

impl ANewDefault for ASet {
    fn new_default() -> Self {
        crate::aset!()
    }
}

impl ANewDefault for ASer {
    fn new_default() -> Self {
        ASer::default()
    }
}

impl<'a> ANewDefault for AInvoker<'a> {
    fn new_default() -> Self {
        AInvoker::default()
    }
}

impl ANewDefault for AFwdPolicy {
    fn new_default() -> Self {
        AFwdPolicy::default()
    }
}

pub trait APolicyExt<P: Policy> {
    fn default_ser(&self) -> P::Ser;

    fn default_set(&self) -> P::Set;

    fn default_inv<'a>(&self) -> P::Inv<'a>;
}

impl<Ser, Chk> APolicyExt<FwdPolicy<ASet, Ser, Chk>> for FwdPolicy<ASet, Ser, Chk>
where
    Ser: Default + 'static,
    Chk: SetChecker<ASet>,
{
    fn default_set(&self) -> ASet {
        crate::aset!()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <FwdPolicy<ASet, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<ASet, Ser>::default()
    }
}

impl<Ser, Chk> APolicyExt<PrePolicy<ASet, Ser, Chk>> for PrePolicy<ASet, Ser, Chk>
where
    Ser: Default + 'static,
    Chk: SetChecker<ASet>,
{
    fn default_set(&self) -> ASet {
        crate::aset!()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <PrePolicy<ASet, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<ASet, Ser>::default()
    }
}

impl<Ser, Chk> APolicyExt<DelayPolicy<ASet, Ser, Chk>> for DelayPolicy<ASet, Ser, Chk>
where
    Ser: Default + 'static,
    Chk: SetChecker<ASet>,
{
    fn default_set(&self) -> ASet {
        crate::aset!()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <DelayPolicy<ASet, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<ASet, Ser>::default()
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

impl<'a> Default for APolicyParser<'a, AFwdPolicy> {
    fn default() -> Self {
        let set = crate::aset!();
        let ser = ASer::default();
        let inv = Invoker::default();

        Self(PolicyParser::new_with(AFwdPolicy::default(), set, inv, ser))
    }
}

impl<'a> Default for APolicyParser<'a, ADelayPolicy> {
    fn default() -> Self {
        let set = crate::aset!();
        let ser = ASer::default();
        let inv = Invoker::default();

        Self(PolicyParser::new_with(
            ADelayPolicy::default(),
            set,
            inv,
            ser,
        ))
    }
}

impl<'a> Default for APolicyParser<'a, APrePolicy> {
    fn default() -> Self {
        let set = crate::aset!();
        let ser = ASer::default();
        let inv = Invoker::default();

        Self(PolicyParser::new_with(APrePolicy::default(), set, inv, ser))
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
}
