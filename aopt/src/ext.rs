use crate::opt::AOpt;
use crate::opt::BoolCreator;
use crate::opt::CmdCreator;
use crate::opt::Creator;
use crate::opt::FltCreator;
use crate::opt::IntCreator;
use crate::opt::MainCreator;
use crate::opt::OptConfig;
use crate::opt::PosCreator;
use crate::opt::StrCreator;
use crate::opt::StrParser;
use crate::opt::UintCreator;
use crate::policy::FwdPolicy;
use crate::policy::PrePolicy;
use crate::ser::CheckService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::UsrValService;
use crate::ser::ValService;
use crate::set::OptSet;
use crate::Error;
use crate::RawVal;
use crate::Uid;

pub mod ctx;
pub mod ser;

/// Some convenient function access the [`Service`](crate::ser::Service) in [`Services`].
pub trait ServicesExt {
    fn ser_val(&self) -> Result<&ValService, Error>;

    fn ser_val_mut(&mut self) -> Result<&mut ValService, Error>;

    fn ser_usrval(&self) -> Result<&UsrValService, Error>;

    fn ser_usrval_mut(&mut self) -> Result<&mut UsrValService, Error>;

    fn ser_invoke<S: 'static>(&self) -> Result<&InvokeService<S>, Error>;

    fn ser_invoke_mut<S: 'static>(&mut self) -> Result<&mut InvokeService<S>, Error>;

    fn ser_rawval<T: 'static>(&self) -> Result<&RawValService<T>, Error>;

    fn ser_rawval_mut<T: 'static>(&mut self) -> Result<&mut RawValService<T>, Error>;

    fn ser_check<S: 'static>(&self) -> Result<&CheckService<S>, Error>;
}

pub trait ServicesValExt<T: 'static> {
    fn val(uid: Uid, ser: &Services) -> Result<&T, Error>;

    fn val_mut(uid: Uid, ser: &mut Services) -> Result<&mut T, Error>;

    fn vals(uid: Uid, ser: &Services) -> Result<&Vec<T>, Error>;

    fn vals_mut(uid: Uid, ser: &mut Services) -> Result<&mut Vec<T>, Error>;
}

pub trait ServicesRawValExt<T: 'static> {
    fn raw_val(uid: Uid, ser: &Services) -> Result<&T, Error>;

    fn raw_val_mut(uid: Uid, ser: &mut Services) -> Result<&mut T, Error>;

    fn raw_vals(uid: Uid, ser: &Services) -> Result<&Vec<T>, Error>;

    fn raw_vals_mut(uid: Uid, ser: &mut Services) -> Result<&mut Vec<T>, Error>;
}

pub trait ServicesUsrValExt<T: 'static> {
    fn usr_val(ser: &Services) -> Result<&T, Error>;

    fn usr_val_mut(ser: &mut Services) -> Result<&mut T, Error>;
}

pub type ACreator = Box<dyn Creator<Opt = AOpt, Config = OptConfig, Error = Error>>;

pub type ASet = OptSet<StrParser, ACreator>;

pub type ASer = Services;

pub type AFwdPolicy = FwdPolicy<ASet>;

pub type APrePolicy = PrePolicy<ASet>;

impl AFwdPolicy {
    /// Get default [`ASet`] for forward policy.
    pub fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    /// Get default [`ASer`] for forward policy.
    pub fn default_ser(&self) -> ASer {
        aser_with_default_service::<ASet>()
    }
}

/// Return an [`Set`](crate::set::Set) with default prefix `-` and `--`,
/// and below [`Creator`]s:
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
