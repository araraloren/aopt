use crate::ctx::Invoker;
use crate::opt::creator::BuiltInCtor;
use crate::opt::AOpt;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::StrParser;
use crate::parser::DefaultSetChecker;
use crate::parser::DelayPolicy;
use crate::parser::FwdPolicy;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::PrePolicy;
use crate::prelude::SetChecker;
use crate::ser::AppServices;
use crate::set::OptSet;
use crate::set::PrefixOptValidator;
use crate::Error;

pub mod ctx;
pub mod ser;

pub trait APolicyExt<P: Policy> {
    fn default_ser(&self) -> P::Ser;

    fn default_set(&self) -> P::Set;

    fn default_inv<'a>(&self) -> P::Inv<'a>;
}

pub type ACreator = Creator<AOpt, OptConfig, Error>;

pub type ASet = OptSet<StrParser, ACreator, PrefixOptValidator>;

pub type ASer = AppServices;

pub type AFwdPolicy = FwdPolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type APrePolicy = PrePolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type ADelayPolicy = DelayPolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type AFwdParser<'a> = Parser<'a, AFwdPolicy>;

pub type APreParser<'a> = Parser<'a, APrePolicy>;

pub type ADelayParser<'a> = Parser<'a, ADelayPolicy>;

impl<Ser, Chk> APolicyExt<FwdPolicy<ASet, Ser, Chk>> for FwdPolicy<ASet, Ser, Chk>
where
    Ser: Default + 'static,
    Chk: SetChecker<ASet>,
{
    fn default_set(&self) -> ASet {
        aset_with_default_creators()
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
        aset_with_default_creators()
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
        aset_with_default_creators()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <DelayPolicy<ASet, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<ASet, Ser>::default()
    }
}

/// Return an [`ASet`](crate::ext::ASet) with below creators:
///
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
pub fn aset_with_default_creators() -> ASet {
    ASet::default()
        .with_creator(Creator::fallback())
        .with_creator(Creator::from(BuiltInCtor::Int))
        .with_creator(Creator::from(BuiltInCtor::Bool))
        .with_creator(Creator::from(BuiltInCtor::Flt))
        .with_creator(Creator::from(BuiltInCtor::Str))
        .with_creator(Creator::from(BuiltInCtor::Uint))
        .with_creator(Creator::from(BuiltInCtor::Cmd))
        .with_creator(Creator::from(BuiltInCtor::Pos))
        .with_creator(Creator::from(BuiltInCtor::Main))
        .with_creator(Creator::from(BuiltInCtor::Any))
        .with_creator(Creator::from(BuiltInCtor::Raw))
}
