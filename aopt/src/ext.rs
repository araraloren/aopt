use crate::map::ErasedTy;
use crate::opt::AOpt;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::StrParser;
use crate::parser::DelayPolicy;
use crate::parser::FwdPolicy;
use crate::parser::Parser;
use crate::parser::PrePolicy;
use crate::ser::CheckService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::UsrValService;
use crate::ser::ValService;
use crate::set::OptSet;
use crate::set::Set;
use crate::Error;
use crate::RawVal;
use crate::Uid;

pub mod ctx;
pub mod ser;

/// Some convenient function access the [`Service`](crate::ser::Service) in [`Services`].
pub trait ServicesExt {
    /// Get [`ValService`] reference.
    fn ser_val(&self) -> Result<&ValService, Error>;

    /// Get [`ValService`] mutable reference.
    fn ser_val_mut(&mut self) -> Result<&mut ValService, Error>;

    /// Get [`UsrValService`] reference.
    fn ser_usrval(&self) -> Result<&UsrValService, Error>;

    /// Get [`UsrValService`] mutable reference.
    fn ser_usrval_mut(&mut self) -> Result<&mut UsrValService, Error>;

    /// Get [`InvokeService`] reference.
    fn ser_invoke<S: Set + 'static>(&self) -> Result<&InvokeService<S>, Error>;

    /// Get [`InvokeService`] mutable reference.
    fn ser_invoke_mut<S: Set + 'static>(&mut self) -> Result<&mut InvokeService<S>, Error>;

    /// Get [`RawValService`] reference.
    fn ser_rawval<T: ErasedTy>(&self) -> Result<&RawValService<T>, Error>;

    /// Get [`RawValService`] mutable reference.
    fn ser_rawval_mut<T: ErasedTy>(&mut self) -> Result<&mut RawValService<T>, Error>;

    /// Get [`CheckService`] reference.
    fn ser_check<S: Set + 'static>(&self) -> Result<&CheckService<S>, Error>;
}

pub trait ServicesValExt {
    /// Get the last value reference of option `uid` from [`ValService`].
    fn sve_val<T: ErasedTy>(&self, uid: Uid) -> Result<&T, Error>;

    /// Get the last value mutable reference of option `uid` from [`ValService`].
    fn sve_val_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut T, Error>;

    /// Take last value of option `uid` from [`ValService`].
    fn sve_take_val<T: ErasedTy>(&mut self, uid: Uid) -> Result<T, Error>;

    /// Get the values reference of option `uid` from [`ValService`].
    fn sve_vals<T: ErasedTy>(&self, uid: Uid) -> Result<&Vec<T>, Error>;

    /// Get the values mutable reference of option `uid` from [`ValService`].
    fn sve_vals_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error>;

    /// Take the values of option `uid` from [`ValService`].
    fn sve_take_vals<T: ErasedTy>(&mut self, uid: Uid) -> Result<Vec<T>, Error>;

    /// Apply filter on the values of option from [`ValService`].
    /// The `F` should return true if you want remove the element.
    fn sve_filter<T: ErasedTy>(
        &mut self,
        uid: Uid,
        f: impl FnMut(&T) -> bool,
    ) -> Result<Vec<T>, Error>;

    /// Get the user value reference of option `uid` from [`UsrValService`].
    fn sve_usrval<T: ErasedTy>(&self) -> Result<&T, Error>;

    /// Get the user value mutable reference of option `uid` from [`UsrValService`].
    fn sve_usrval_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error>;

    /// Take the user value of option `uid` from [`UsrValService`].
    fn sve_take_usrval<T: ErasedTy>(&mut self) -> Result<T, Error>;

    /// Get the raw value reference of option `uid` from [`RawValService`].
    fn sve_rawval(&self, uid: Uid) -> Result<&RawVal, Error>;

    /// Get the raw value mutable reference of option `uid` from [`RawValService`].
    fn sve_rawval_mut(&mut self, uid: Uid) -> Result<&mut RawVal, Error>;

    /// Get the raw values reference of option `uid` from [`RawValService`].
    fn sve_rawvals(&self, uid: Uid) -> Result<&Vec<RawVal>, Error>;

    /// Get the raw values mutable reference of option `uid` from [`RawValService`].
    fn sve_rawvals_mut(&mut self, uid: Uid) -> Result<&mut Vec<RawVal>, Error>;
}

pub trait APolicyExt<I: crate::set::Set> {
    fn default_ser(&self) -> ASer;

    fn default_set(&self) -> I;
}

pub type ACreator = Creator<AOpt, OptConfig, Error>;

pub type ASet = OptSet<StrParser, ACreator>;

pub type ASer = Services;

pub type AFwdPolicy = FwdPolicy<ASet>;

pub type APrePolicy = PrePolicy<ASet>;

pub type ADelayPolicy = DelayPolicy<ASet>;

pub type AFwdParser = Parser<AFwdPolicy>;

pub type APreParser = Parser<APrePolicy>;

pub type ADelayParser = Parser<ADelayPolicy>;

impl APolicyExt<ASet> for AFwdPolicy {
    /// Get default [`ASet`] for forward policy.
    fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    /// Get default [`ASer`] for forward policy.
    fn default_ser(&self) -> ASer {
        aser_with_default_service::<ASet>()
    }
}

impl APolicyExt<ASet> for APrePolicy {
    /// Get default [`ASet`] for forward policy.
    fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    /// Get default [`ASer`] for forward policy.
    fn default_ser(&self) -> ASer {
        aser_with_default_service::<ASet>()
    }
}

impl APolicyExt<ASet> for ADelayPolicy {
    /// Get default [`ASet`] for forward policy.
    fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    /// Get default [`ASer`] for forward policy.
    fn default_ser(&self) -> ASer {
        aser_with_default_service::<ASet>()
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
pub fn aset_with_default_creators() -> ASet {
    ASet::default()
        .with_prefix("--")
        .with_prefix("-")
        .with_creator(Creator::int())
        .with_creator(Creator::bool())
        .with_creator(Creator::uint())
        .with_creator(Creator::flt())
        .with_creator(Creator::str())
        .with_creator(Creator::cmd())
        .with_creator(Creator::main())
        .with_creator(Creator::pos())
}

/// Return an [`Services`] with below [`Service`](crate::ser::Service)s:
///
/// * [`CheckService`]
/// * [`UsrValService`]
/// * [`InvokeService`]
/// * [`RawValService`]
/// * [`ValService`]
pub fn aser_with_default_service<S: Set + 'static>() -> Services {
    Services::default()
        .with(CheckService::<S>::new())
        .with(UsrValService::new())
        .with(InvokeService::<S>::new())
        .with(RawValService::<RawVal>::new())
        .with(ValService::new())
}
