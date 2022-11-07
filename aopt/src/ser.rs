pub(crate) mod check;
pub(crate) mod data;
pub(crate) mod invoke;
pub(crate) mod rawval;
pub(crate) mod value;

pub use self::check::CheckService;
pub use self::data::DataService;
pub use self::invoke::InvokeService;
pub use self::invoke::SerRegister;
pub use self::rawval::RawValService;
pub use self::value::ValEntry;
pub use self::value::ValService;

use crate::map::AnyMap;
use crate::Error;
use crate::Str;

pub trait Service {
    fn service_name() -> Str;
}

/// Services keep different type [`Service`]s in a map.
///
/// # Examples
/// ```rust
/// # use aopt::Result;
/// # use aopt::prelude::*;
/// # fn main() -> Result<()> {
///     #[derive(Debug, PartialEq)]
///     struct MyVec(pub Vec<i32>);
///
///     impl Service for MyVec {
///         fn service_name() -> Str {
///             astr("VecService")
///         }
///     }
///
///     #[derive(Debug, PartialEq)]
///     struct I32(i32);
///
///     impl Service for I32 {
///         fn service_name() -> Str {
///             astr("I32Service")
///         }
///     }
///
///     let mut services = Services::new().with(MyVec(vec![42i32]));
///
///     // get value from of service
///     assert_eq!(services.ser::<MyVec>()?.0[0], 42);
///     // modfify the service value
///     services.ser_mut::<MyVec>()?.0.push(18);
///     // check the value of MyVec
///     assert_eq!(services.ser::<MyVec>()?.0[1], 18);
///
///     // register a new service
///     services.reg(I32(42));
///     assert!(services.has::<I32>());
///
///     // unregister service from
///     services.unreg::<MyVec>();
///     assert!(!services.has::<MyVec>());
///
///     Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct Services(AnyMap);

impl Services {
    pub fn new() -> Self {
        Self(AnyMap::new())
    }

    pub fn with<T: Service + 'static>(mut self, value: T) -> Self {
        self.register(value);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return true if [`Services`] contain a service type T.
    pub fn contain<T: Service + 'static>(&self) -> bool {
        self.0.contain::<T>()
    }

    pub fn get<T: Service + 'static>(&self) -> Option<&T> {
        self.0.get::<T>()
    }

    pub fn get_mut<T: Service + 'static>(&mut self) -> Option<&mut T> {
        self.0.get_mut::<T>()
    }

    pub fn remove<T: Service + 'static>(&mut self) -> Option<T> {
        self.0.remove::<T>()
    }

    pub fn register<T: Service + 'static>(&mut self, value: T) -> Option<T> {
        self.0.insert(value)
    }

    pub fn service<T: Service + 'static>(&self) -> Result<&T, Error> {
        self.get::<T>().ok_or_else(|| {
            Error::raise_error(format!("Unknown type {} for Services", T::service_name()))
        })
    }

    pub fn service_mut<T: Service + 'static>(&mut self) -> Result<&mut T, Error> {
        self.get_mut::<T>().ok_or_else(|| {
            Error::raise_error(format!("Unknown type {} for Services", T::service_name(),))
        })
    }

    pub fn take<T: Service + 'static>(&mut self) -> Result<T, Error> {
        self.remove::<T>().ok_or_else(|| {
            Error::raise_error(format!("Unknown type {} for Services", T::service_name(),))
        })
    }
}