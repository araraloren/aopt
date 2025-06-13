pub mod args;
pub mod ctx;
pub mod err;
pub mod map;
pub mod opt;
pub mod parser;
pub mod str;
pub mod value;

pub type Uid = u64;

pub use err::Error;

pub type HashMap<K, V> = ahash::HashMap<K, V>;

#[cfg(feature = "sync")]
pub type ARef<T> = std::sync::Arc<T>;
#[cfg(not(feature = "sync"))]
pub type ARef<T> = std::rc::Rc<T>;

#[cfg(feature = "log")]
pub use tracing::trace;
#[cfg(not(feature = "log"))]
#[macro_use]
pub mod log {
    #[macro_export]
    macro_rules! trace {
        ($($arg:tt)*) => {};
    }
}

/// Get the [`TypeId`](std::any::TypeId) of type `T`.
pub(crate) fn typeid<T: ?Sized + 'static>() -> std::any::TypeId {
    std::any::TypeId::of::<T>()
}
