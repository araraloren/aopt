pub mod args;
pub mod ctx;
pub mod err;
pub mod ext;
pub mod map;
pub mod opt;
pub mod policy;
pub mod proc;
pub mod raw;
pub mod ser;
pub mod set;
pub mod str;

pub type Uid = u64;
pub type HashMap<K, V> = ahash::HashMap<K, V>;
pub type RawVal = raw::RawVal;
cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub type Arc<T> = std::sync::Arc<T>;
    }
    else {
        pub type Arc<T> = std::rc::Rc<T>;
    }
}

pub use crate::err::Error;
pub use crate::err::Result;
pub use crate::str::astr;
pub use crate::str::Str;
pub use crate::str::StrJoin;

use std::any::TypeId;
/// Get the [`TypeId`](std::any::TypeId) of type `T`.
pub(crate) fn typeid<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

pub mod prelude {
    pub use crate::args::Args;
    pub use crate::ctx::wrap_handler;
    pub use crate::ctx::Callback;
    pub use crate::ctx::Callbacks;
    pub use crate::ctx::Ctx;
    pub use crate::ctx::Extract;
    pub use crate::ctx::Handler;
    pub use crate::ctx::Store;
    pub use crate::ext::*;
    pub use crate::opt::AOpt;
    pub use crate::opt::Action;
    pub use crate::opt::Assoc;
    pub use crate::opt::BoolCreator;
    pub use crate::opt::CmdCreator;
    pub use crate::opt::Config;
    pub use crate::opt::ConfigValue;
    pub use crate::opt::ConstrctInfo;
    pub use crate::opt::Creator;
    pub use crate::opt::FltCreator;
    pub use crate::opt::Help;
    pub use crate::opt::Index;
    pub use crate::opt::Information;
    pub use crate::opt::IntCreator;
    pub use crate::opt::MainCreator;
    pub use crate::opt::Opt;
    pub use crate::opt::OptConfig;
    pub use crate::opt::OptParser;
    pub use crate::opt::PosCreator;
    pub use crate::opt::RawValParser;
    pub use crate::opt::RawValValidator;
    pub use crate::opt::Serde;
    pub use crate::opt::StrCreator;
    pub use crate::opt::StrParser;
    pub use crate::opt::Style;
    pub use crate::opt::UintCreator;
    pub use crate::opt::ValInitialize;
    pub use crate::opt::ValInitiator;
    pub use crate::opt::ValValidator;
    pub use crate::policy::Forward;
    pub use crate::policy::Policy;
    pub use crate::proc::Match;
    pub use crate::proc::NOAMatch;
    pub use crate::proc::NOAProcess;
    pub use crate::proc::OptMatch;
    pub use crate::proc::OptProcess;
    pub use crate::proc::Process;
    pub use crate::ser::CheckService;
    pub use crate::ser::InvokeService;
    pub use crate::ser::RawValService;
    pub use crate::ser::Service;
    pub use crate::ser::Services;
    pub use crate::ser::UsrValService;
    pub use crate::ser::ValEntry;
    pub use crate::ser::ValService;
    pub use crate::set::Commit;
    pub use crate::set::Filter;
    pub use crate::set::FilterMatcher;
    pub use crate::set::FilterMut;
    pub use crate::set::OptSet;
    pub use crate::set::Pre;
    pub use crate::set::Set;
    pub use crate::set::SetExt;
    pub use crate::Uid;
}
