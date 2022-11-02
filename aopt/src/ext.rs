use crate::opt::AOpt;
use crate::opt::BoolCreator;
use crate::opt::Creator;
use crate::opt::IntCreator;
use crate::opt::OptConfig;
use crate::opt::StrParser;
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

pub type ACreator = Box<dyn Creator<Opt = AOpt, Config = OptConfig, Error = Error>>;

pub type ASet = OptSet<StrParser, ACreator>;

pub type AForward = Forward<ASet>;

pub(crate) fn aset_with_default_creators() -> ASet {
    ASet::default()
        .with_prefix("--")
        .with_prefix("-")
        .with_creator(IntCreator::boxed())
        .with_creator(BoolCreator::boxed())
}

pub(crate) fn services_with_default_service<S: 'static>() -> Services {
    Services::default()
        .with(CheckService::<S>::new())
        .with(DataService::new())
        .with(InvokeService::<S>::new())
        .with(RawValService::<RawVal>::new())
        .with(ValService::new())
}

impl AForward {
    pub fn default_set(&self) -> ASet {
        aset_with_default_creators()
    }

    pub fn default_ser(&self) -> Services {
        services_with_default_service::<ASet>()
    }
}
