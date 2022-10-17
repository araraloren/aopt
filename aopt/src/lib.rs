pub mod aopt;
pub mod arg;
pub mod ctx;
pub mod err;
pub mod ext;
pub(crate) mod map;
pub mod opt;
pub mod policy;
pub mod proc;
pub mod ser;
pub mod set;
pub mod sim;
pub mod str;

pub type Uid = u64;
pub type HashMap<K, V> = ahash::HashMap<K, V>;
pub type RawVal = std::ffi::OsString;
pub type Arc<T> = std::rc::Rc<T>;
pub use self::err::Error;
pub use self::err::Result;
pub use self::str::astr;
pub use self::str::Str;
pub use self::str::StrJoin;

pub(crate) fn typeid<T>() -> std::any::TypeId
where
    T: 'static,
{
    std::any::TypeId::of::<T>()
}

pub mod prelude {
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
    pub use crate::ctx;
    pub use crate::ctx::Callback;
    pub use crate::ctx::Callbacks;
    pub use crate::ctx::Ctx;
    pub use crate::ctx::ExtractCtx;
    pub use crate::ctx::Handler;
    pub use crate::ext::APolicyExt;
    pub use crate::ext::AServiceExt;
    pub use crate::ext::ASetConfigExt;
    pub use crate::ext::ASetExt;
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
    pub use crate::policy::CtxSaver;
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
    pub use crate::ser::CheckService;
    pub use crate::ser::DataService;
    pub use crate::ser::InvokeService;
    pub use crate::ser::RawValService;
    pub use crate::ser::Service;
    pub use crate::ser::Services;
    pub use crate::ser::ServicesExt;
    pub use crate::set::Commit;
    pub use crate::set::OptSet;
    pub use crate::set::PreSet;
    pub use crate::set::Set;
    pub use crate::set::SetExt;
    pub use crate::sim::SimCtor;
    pub use crate::sim::SimOpt;
    pub use crate::sim::SimSer;
    pub use crate::sim::SimSet;
    pub use crate::Str;
    pub use crate::Uid;
}
