pub mod args;
pub mod ctx;
pub mod err;
pub mod ext;
pub mod map;
pub mod opt;
pub mod policy;
pub mod proc;
pub mod ser;
pub mod set;
pub mod str;

pub type Uid = u64;
pub type HashMap<K, V> = ahash::HashMap<K, V>;
cfg_if::cfg_if! {
    if #[cfg(feature = "utf8")] {
        pub type RawVal = String;
        pub type RawValRef = str;
    }
    else {
        pub type RawVal = std::ffi::OsString;
        pub type RawValRef = std::ffi::OsStr;
    }
}
cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
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
    pub use crate::ser::Service;
    pub use crate::ser::Services;
}
