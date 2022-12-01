use crate::opt::AOpt;
use crate::opt::BoolCreator;
use crate::opt::CmdCreator;
use crate::opt::FltCreator;
use crate::opt::IntCreator;
use crate::opt::MainCreator;
use crate::opt::OptConfig;
use crate::opt::PosCreator;
use crate::opt::StrCreator;
use crate::opt::StrParser;
use crate::opt::UintCreator;
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
use crate::set::Ctor;
use crate::set::OptSet;
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
    fn ser_invoke<S: 'static>(&self) -> Result<&InvokeService<S>, Error>;

    /// Get [`InvokeService`] mutable reference.
    fn ser_invoke_mut<S: 'static>(&mut self) -> Result<&mut InvokeService<S>, Error>;

    /// Get [`RawValService`] reference.
    fn ser_rawval<T: 'static>(&self) -> Result<&RawValService<T>, Error>;

    /// Get [`RawValService`] mutable reference.
    fn ser_rawval_mut<T: 'static>(&mut self) -> Result<&mut RawValService<T>, Error>;

    /// Get [`CheckService`] reference.
    fn ser_check<S: 'static>(&self) -> Result<&CheckService<S>, Error>;
}

pub trait ServicesValExt<T: 'static> {
    /// Get the last value reference of option `uid` from [`ValService`].
    fn sve_val(uid: Uid, ser: &Services) -> Result<&T, Error>;

    /// Get the last value mutable reference of option `uid` from [`ValService`].
    fn sve_val_mut(uid: Uid, ser: &mut Services) -> Result<&mut T, Error>;

    /// Take last value of option `uid` from [`ValService`].
    fn sve_take_val(uid: Uid, ser: &mut Services) -> Result<T, Error>;

    /// Get the values reference of option `uid` from [`ValService`].
    fn sve_vals(uid: Uid, ser: &Services) -> Result<&Vec<T>, Error>;

    /// Get the values mutable reference of option `uid` from [`ValService`].
    fn sve_vals_mut(uid: Uid, ser: &mut Services) -> Result<&mut Vec<T>, Error>;

    /// Take the values of option `uid` from [`ValService`].
    fn sve_take_vals(uid: Uid, ser: &mut Services) -> Result<Vec<T>, Error>;

    /// Apply filter on the values of option from [`ValService`].
    /// The `F` should return true if you want remove the element.
    fn sve_filter<F: FnMut(&T) -> bool>(
        uid: Uid,
        ser: &mut Services,
        f: F,
    ) -> Result<Vec<T>, Error>;

    /// Get the user value reference of option `uid` from [`UsrValService`].
    fn sve_usrval(ser: &Services) -> Result<&T, Error>;

    /// Get the user value mutable reference of option `uid` from [`UsrValService`].
    fn sve_usrval_mut(ser: &mut Services) -> Result<&mut T, Error>;

    /// Take the user value of option `uid` from [`UsrValService`].
    fn sve_take_usrval(ser: &mut Services) -> Result<T, Error>;
}

pub trait ServicesRawValExt<T: 'static> {
    fn srve_val(uid: Uid, ser: &Services) -> Result<&T, Error>;

    fn srve_val_mut(uid: Uid, ser: &mut Services) -> Result<&mut T, Error>;

    fn srve_vals(uid: Uid, ser: &Services) -> Result<&Vec<T>, Error>;

    fn srve_vals_mut(uid: Uid, ser: &mut Services) -> Result<&mut Vec<T>, Error>;
}

pub trait APolicyExt<I: crate::set::Set> {
    fn default_ser(&self) -> ASer;

    fn default_set(&self) -> I;
}

pub type ACreator = Box<dyn Ctor<Opt = AOpt, Config = OptConfig, Error = Error>>;

pub type ASet = OptSet<StrParser, ACreator>;

pub type ASer = Services;

pub type AFwdPolicy = FwdPolicy<ASet>;

pub type APrePolicy = PrePolicy<ASet>;

pub type ADelayPolicy = DelayPolicy<ASet>;

pub type AFwdParser = Parser<AFwdPolicy, ASet, bool>;

pub type APreParser = Parser<APrePolicy, ASet, Vec<RawVal>>;

pub type ADelayParser = Parser<ADelayPolicy, ASet, bool>;

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
/// and below [`Ctor`]s:
///
/// * [`IntCreator`]
/// * [`BoolCreator`]
/// * [`UintCreator`]
/// * [`StrCreator`]
/// * [`FltCreator`]
/// * [`CmdCreator`]
/// * [`PosCreator`]
/// * [`MainCreator`]
pub fn aset_with_default_creators() -> ASet {
    ASet::default()
        .with_prefix("--")
        .with_prefix("-")
        .with_creator(IntCreator::boxed())
        .with_creator(BoolCreator::boxed())
        .with_creator(UintCreator::boxed())
        .with_creator(StrCreator::boxed())
        .with_creator(FltCreator::boxed())
        .with_creator(CmdCreator::boxed())
        .with_creator(PosCreator::boxed())
        .with_creator(MainCreator::boxed())
}

/// Return an [`Services`] with below [`Service`](crate::ser::Service)s:
///
/// * [`CheckService`]
/// * [`UsrValService`]
/// * [`InvokeService`]
/// * [`RawValService`]
/// * [`ValService`]
pub fn aser_with_default_service<S: 'static>() -> Services {
    Services::default()
        .with(CheckService::<S>::new())
        .with(UsrValService::new())
        .with(InvokeService::<S>::new())
        .with(RawValService::<RawVal>::new())
        .with(ValService::new())
}
