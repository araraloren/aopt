pub mod aopt;
pub mod arg;
pub mod ctx;
pub mod err;
pub(crate) mod map;
pub mod opt;
pub mod policy;
pub mod proc;
pub mod ser;
pub mod set;
pub mod str;
//pub mod data;

use aopt::ACreator;
use aopt::AOpt;
use opt::OptConfig;
use opt::OptStringParser;
use ser::Services;
use set::ASetExt;
use set::OptSet;

pub type Uid = u64;
pub type HashMap<K, V> = ahash::HashMap<K, V>;
pub type SimpleOpt = Box<dyn AOpt>;
pub type SimpleCreator = Box<dyn ACreator<Opt = SimpleOpt, Config = OptConfig>>;
pub type SimpleServices = Services;
pub type SimpleSet = OptSet<SimpleOpt, OptStringParser, SimpleCreator>;

pub trait DefaultSetConfig {
    fn with_default_prefix(self) -> Self;

    fn with_default_creator(self) -> Self;
}

impl DefaultSetConfig for SimpleSet {
    fn with_default_prefix(mut self) -> Self {
        self = self.with_prefix("--").with_prefix("-");
        self
    }

    fn with_default_creator(mut self) -> Self {
        self = self
            .with_creator(aopt::BoolCreator::boxed())
            .with_creator(aopt::FltCreator::boxed())
            .with_creator(aopt::IntCreator::boxed())
            .with_creator(aopt::StrCreator::boxed())
            .with_creator(aopt::UintCreator::boxed())
            .with_creator(aopt::CmdCreator::boxed())
            .with_creator(aopt::PosCreator::boxed())
            .with_creator(aopt::MainCreator::boxed());
        self
    }
}

impl ASetExt for SimpleSet {
    fn new_default() -> Self {
        Self::default().with_default_prefix().with_default_creator()
    }
}

pub use self::err::Error;
pub use self::err::Result;
pub use self::str::astr;
pub use self::str::Str;
pub use self::str::StrJoin;

use std::any::TypeId;

pub(crate) fn typeid<T>() -> TypeId
where
    T: 'static,
{
    TypeId::of::<T>()
}

pub mod prelude {
    //pub use crate::data;
    pub use crate::aopt::ACreator;
    pub use crate::aopt::AOpt;
    pub use crate::aopt::BoolCreator;
    pub use crate::aopt::BoolOpt;
    pub use crate::aopt::CmdCreator;
    pub use crate::aopt::CmdOpt;
    pub use crate::aopt::FltCreator;
    pub use crate::aopt::FltOpt;
    pub use crate::aopt::IntCreator;
    pub use crate::aopt::IntOpt;
    pub use crate::aopt::MainCreator;
    pub use crate::aopt::MainOpt;
    pub use crate::aopt::PosCreator;
    pub use crate::aopt::PosOpt;
    pub use crate::aopt::StrCreator;
    pub use crate::aopt::StrOpt;
    pub use crate::aopt::UintCreator;
    pub use crate::aopt::UintOpt;
    pub use crate::arg::Args;
    pub use crate::astr;
    pub use crate::ctx::wrap_callback;
    pub use crate::ctx::Callback;
    pub use crate::ctx::Callbacks;
    pub use crate::ctx::Context;
    pub use crate::ctx::ExtractFromCtx;
    pub use crate::ctx::Handler;
    pub use crate::map::Map;
    pub use crate::map::MapExt;
    pub use crate::map::RcMap;
    pub use crate::map::RcMapExt;
    pub use crate::opt::Alias;
    pub use crate::opt::Config;
    pub use crate::opt::ConfigValue;
    pub use crate::opt::Creator;
    pub use crate::opt::Help;
    pub use crate::opt::Index;
    pub use crate::opt::Information;
    pub use crate::opt::Name;
    pub use crate::opt::Opt;
    pub use crate::opt::OptCallback;
    pub use crate::opt::OptConfig;
    pub use crate::opt::OptConstrctInfo;
    pub use crate::opt::OptHelp;
    pub use crate::opt::OptIndex;
    pub use crate::opt::OptParser;
    pub use crate::opt::OptStringParser;
    pub use crate::opt::OptStyle;
    pub use crate::opt::Optional;
    pub use crate::opt::Prefix;
    pub use crate::policy::APolicyExt;
    pub use crate::policy::ContextSaver;
    pub use crate::policy::DelayPolicy;
    pub use crate::policy::ForwardPolicy;
    pub use crate::policy::Policy;
    pub use crate::policy::PrePolicy;
    pub use crate::proc::Match;
    pub use crate::proc::NOAMatch;
    pub use crate::proc::NOAProcess;
    pub use crate::proc::OptMatch;
    pub use crate::proc::OptProcess;
    pub use crate::proc::Process;
    pub use crate::ser::AServiceExt;
    pub use crate::ser::CheckService;
    pub use crate::ser::DataService;
    pub use crate::ser::InvokeService;
    pub use crate::ser::Service;
    pub use crate::ser::Services;
    pub use crate::ser::ServicesExt;
    pub use crate::ser::ValueService;
    pub use crate::set::ASetExt;
    pub use crate::set::Commit;
    pub use crate::set::OptSet;
    pub use crate::set::Prefixed;
    pub use crate::set::Set;
    pub use crate::set::SetExt;
    pub use crate::DefaultSetConfig;
    pub use crate::SimpleCreator;
    pub use crate::SimpleOpt;
    pub use crate::SimpleServices;
    pub use crate::SimpleSet;
    pub use crate::Str;
    pub use crate::Uid;
}
