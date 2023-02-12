use crate::ctx::Invoker;
use crate::opt::creator::BuiltInCtor;
use crate::opt::AOpt;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::StrParser;
use crate::parser::DelayPolicy;
use crate::parser::FwdPolicy;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::PrePolicy;
use crate::ser::AppServices;
use crate::set::OptSet;
use crate::set::PrefixOptValidator;
use crate::Error;

pub mod ctx;
pub mod ser;

pub trait APolicyExt<P: Policy> {
    fn default_ser(&self) -> P::Ser;

    fn default_set(&self) -> P::Set;

    fn default_inv(&self) -> P::Inv;
}

pub type ACreator = Creator<AOpt, OptConfig, Error>;

pub type ASet = OptSet<StrParser, ACreator, PrefixOptValidator>;

pub type ASer = AppServices;

pub type AFwdPolicy = FwdPolicy<ASet, ASer>;

pub type APrePolicy = PrePolicy<ASet, ASer>;

pub type ADelayPolicy = DelayPolicy<ASet, ASer>;

pub type AFwdParser = Parser<AFwdPolicy>;

pub type APreParser = Parser<APrePolicy>;

pub type ADelayParser = Parser<ADelayPolicy>;

impl APolicyExt<AFwdPolicy> for AFwdPolicy {
    fn default_set(&self) -> <AFwdPolicy as Policy>::Set {
        aset_with_default_creators()
    }

    fn default_ser(&self) -> <AFwdPolicy as Policy>::Ser {
        ASer::default()
    }

    fn default_inv(&self) -> <AFwdPolicy as Policy>::Inv {
        Invoker::<<AFwdPolicy as Policy>::Set, <AFwdPolicy as Policy>::Ser>::default()
    }
}

impl APolicyExt<APrePolicy> for APrePolicy {
    fn default_set(&self) -> <AFwdPolicy as Policy>::Set {
        aset_with_default_creators()
    }

    fn default_ser(&self) -> <AFwdPolicy as Policy>::Ser {
        ASer::default()
    }

    fn default_inv(&self) -> <APrePolicy as Policy>::Inv {
        Invoker::<<APrePolicy as Policy>::Set, <APrePolicy as Policy>::Ser>::default()
    }
}

impl APolicyExt<ADelayPolicy> for ADelayPolicy {
    fn default_set(&self) -> <AFwdPolicy as Policy>::Set {
        aset_with_default_creators()
    }

    fn default_ser(&self) -> <AFwdPolicy as Policy>::Ser {
        ASer::default()
    }

    fn default_inv(&self) -> <ADelayPolicy as Policy>::Inv {
        Invoker::<<ADelayPolicy as Policy>::Set, <ADelayPolicy as Policy>::Ser>::default()
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
