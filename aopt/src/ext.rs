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
use crate::policy::Forward;
use crate::ser::CheckService;
use crate::ser::DataService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::ValService;
use crate::set::OptSet;
use crate::Error;
use crate::RawVal;

pub mod ctx;
pub mod ser;

pub trait ServicesExt {
    fn ser_val(&self) -> Result<&ValService, Error>;

    fn ser_val_mut(&mut self) -> Result<&mut ValService, Error>;

    fn ser_data(&self) -> Result<&DataService, Error>;

    fn ser_data_mut(&mut self) -> Result<&mut DataService, Error>;

    fn ser_invoke<S: 'static>(&self) -> Result<&InvokeService<S>, Error>;

    fn ser_invoke_mut<S: 'static>(&mut self) -> Result<&mut InvokeService<S>, Error>;

    fn ser_rawval<T: 'static>(&self) -> Result<&RawValService<T>, Error>;

    fn ser_rawval_mut<T: 'static>(&mut self) -> Result<&mut RawValService<T>, Error>;
}

pub type ACreator = Box<dyn Creator<Opt = AOpt, Config = OptConfig, Error = Error>>;

pub type ASet = OptSet<StrParser, ACreator>;

pub type AForward = Forward<ASet>;

impl AForward {
    pub fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    pub fn default_ser(&self) -> Services {
        services_with_default_service::<ASet>()
    }
}

pub(crate) fn aset_with_default_creators() -> ASet {
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

pub(crate) fn services_with_default_service<S: 'static>() -> Services {
    Services::default()
        .with(CheckService::<S>::new())
        .with(DataService::new())
        .with(InvokeService::<S>::new())
        .with(RawValService::<RawVal>::new())
        .with(ValService::new())
}
