use crate::opt::AOpt;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::StrParser;
use crate::parser::DelayPolicy;
use crate::parser::FwdPolicy;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::PrePolicy;
use crate::ser::Services;
use crate::set::OptSet;
use crate::set::PrefixOptValidator;
use crate::Error;

pub mod ctx;
pub mod ser;

pub trait APolicyExt<P: Policy> {
    fn default_ser(&self) -> P::Ser;

    fn default_set(&self) -> P::Set;

    fn default_inv(&self) -> P::Inv {
        todo!()
    }
}

pub type ACreator = Creator<AOpt, OptConfig, Error>;

pub type ASet = OptSet<StrParser, ACreator, PrefixOptValidator>;

pub type ASer = Services;

pub type AFwdPolicy = FwdPolicy<ASet, ASer>;

pub type APrePolicy = PrePolicy<ASet, ASer>;

pub type ADelayPolicy = DelayPolicy<ASet, ASer>;

pub type AFwdParser = Parser<AFwdPolicy>;

pub type APreParser = Parser<APrePolicy>;

pub type ADelayParser = Parser<ADelayPolicy>;

impl APolicyExt<AFwdPolicy> for AFwdPolicy {
    /// Get default [`ASet`] for forward policy.
    fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    /// Get default [`ASer`] for forward policy.
    fn default_ser(&self) -> ASer {
        aser_with_default_service()
    }
}

impl APolicyExt<APrePolicy> for APrePolicy {
    /// Get default [`ASet`] for forward policy.
    fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    /// Get default [`ASer`] for forward policy.
    fn default_ser(&self) -> ASer {
        aser_with_default_service()
    }
}

impl APolicyExt<ADelayPolicy> for ADelayPolicy {
    /// Get default [`ASet`] for forward policy.
    fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    /// Get default [`ASer`] for forward policy.
    fn default_ser(&self) -> ASer {
        aser_with_default_service()
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
    ASet::default()
        .with_creator(Creator::int())
        .with_creator(Creator::bool())
        .with_creator(Creator::uint())
        .with_creator(Creator::flt())
        .with_creator(Creator::str())
        .with_creator(Creator::cmd())
        .with_creator(Creator::main())
        .with_creator(Creator::pos())
        .with_creator(Creator::any())
}

/// Return an [`Services`] with below [`Service`](crate::ser::Service)s:
///
/// * [`CheckService`]
/// * [`UsrValService`]
/// * [`InvokeService`]
/// * [`RawValService`]
/// * [`ValService`]
pub fn aser_with_default_service() -> Services {
    Services::default()
}
