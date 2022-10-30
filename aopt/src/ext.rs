use crate::opt::AOpt;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::StrParser;
use crate::ser::CheckService;
use crate::ser::DataService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::ValService;
use crate::RawVal;
use crate::Error;
use crate::set::OptSet;

pub mod ctx;
pub mod ser;

pub type ACreator = Box<dyn Creator<Opt = AOpt, Config = OptConfig, Error = Error>>;

pub type ASet = OptSet<AOpt, StrParser, ACreator>;

pub(crate) fn services_with_default_service<S: 'static>() -> Services {
    Services::default()
        .with(CheckService::<S>::new())
        .with(DataService::new())
        .with(InvokeService::<S>::new())
        .with(RawValService::<RawVal>::new())
        .with(ValService::new())
}
