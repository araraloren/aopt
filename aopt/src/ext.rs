use crate::ctx::Invoker;
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

/// Return an [`Set`](crate::set::Set) with default prefix `-` and `--`,
/// and below creators:
///
/// * [`int`](Creator::int)
/// * [`bool`](Creator::bool)
/// * [`uint`](Creator::uint)
/// * [`str`](Creator::str)
/// * [`flt`](Creator::flt)
/// * [`cmd`](Creator::cmd)
/// * [`pos`](Creator::pos)
/// * [`main`](Creator::main)
/// * [`any`](Creator::any)
pub fn aset_with_default_creators() -> ASet {
    ASet::default().with_creator(Creator::fallback())
}
