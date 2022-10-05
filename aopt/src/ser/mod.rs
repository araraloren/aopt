pub(crate) mod check;
pub(crate) mod invoke;
pub(crate) mod noa;
pub(crate) mod value;

pub use self::check::CheckService;
pub use self::invoke::InvokeService;
use self::noa::NOAService;
pub use self::value::ValueService;

pub type DataService = RcMap;

use std::fmt::Debug;

use crate::astr;
use crate::ctx::ExtractFromCtx;
use crate::ctx::Handler;
use crate::map::Map;
use crate::map::RcMap;
use crate::opt::Opt;
use crate::set::Set;
use crate::typeid;
use crate::Error;
use crate::Str;
use crate::Uid;

pub trait Service {
    fn service_name() -> Str;
}

impl Service for DataService {
    fn service_name() -> Str {
        astr("DataService")
    }
}

/// Services keep different type [`Service`]s in a map.
///
/// # Examples
/// ```rust
/// # use aopt_stable::Result;
/// # use aopt_stable::prelude::*;
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
///     assert_eq!(services.get_service::<MyVec>()?.0[0], 42);
///     // modfify the service value
///     services.get_service_mut::<MyVec>()?.0.push(18);
///     // check the value of MyVec
///     assert_eq!(services.get_service::<MyVec>()?.0[1], 18);
///
///     // register a new service
///     services.register(I32(42));
///     assert!(services.contain::<I32>());
///
///     // unregister service from
///     services.unregister::<MyVec>();
///     assert!(!services.contain::<MyVec>());
///
///     Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct Services(Map);

impl Services {
    pub fn new() -> Self {
        Self(Map::new())
    }

    pub fn with<T>(mut self, value: T) -> Self
    where
        T: Service + 'static,
    {
        self.register(value);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return true if [`Services`] contain a service type T.
    pub fn contain<T>(&self) -> bool
    where
        T: Service + 'static,
    {
        self.0.contain::<T>()
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: Service + 'static,
    {
        self.0.get::<T>()
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Service + 'static,
    {
        self.0.get_mut::<T>()
    }

    pub fn unregister<T>(&mut self) -> Option<T>
    where
        T: Service + 'static,
    {
        self.0.remove::<T>()
    }

    pub fn register<T>(&mut self, value: T) -> Option<T>
    where
        T: Service + 'static,
    {
        self.0.insert(value)
    }
}

pub trait ServicesExt {
    fn get_service<T>(&self) -> Result<&T, Error>
    where
        T: Service + 'static;

    fn get_service_mut<T>(&mut self) -> Result<&mut T, Error>
    where
        T: Service + 'static;

    fn take_service<T>(&mut self) -> Result<T, Error>
    where
        T: Service + 'static;
}

impl ServicesExt for Services {
    fn get_service<T>(&self) -> Result<&T, Error>
    where
        T: Service + 'static,
    {
        debug_assert!(self.contain::<T>(), "Unknown type for Services");
        self.get::<T>().ok_or_else(|| {
            Error::raise_error(format!(
                "Unknown type {} for Services: {:?}",
                T::service_name(),
                typeid::<T>()
            ))
        })
    }

    fn get_service_mut<T>(&mut self) -> Result<&mut T, Error>
    where
        T: Service + 'static,
    {
        debug_assert!(self.contain::<T>(), "Unknown type for Services");
        self.get_mut::<T>().ok_or_else(|| {
            Error::raise_error(format!(
                "Unknown type {} for Services: {:?}",
                T::service_name(),
                typeid::<T>()
            ))
        })
    }

    fn take_service<T>(&mut self) -> Result<T, Error>
    where
        T: Service + 'static,
    {
        debug_assert!(self.contain::<T>(), "Unknown type for Services");
        self.unregister::<T>().ok_or_else(|| {
            Error::raise_error(format!(
                "Unknown type {} for Services: {:?}",
                T::service_name(),
                typeid::<T>()
            ))
        })
    }
}

pub trait AServiceExt<S, V>
where
    S: Set + 'static,
    V: From<Str> + 'static,
    S::Opt: Opt,
{
    fn new_default() -> Self;

    fn noa_service(&self) -> &NOAService;

    fn noa_service_mut(&mut self) -> &mut NOAService;

    fn data_service(&self) -> &DataService;

    fn data_service_mut(&mut self) -> &mut DataService;

    fn value_service(&self) -> &ValueService<V>;

    fn value_service_mut(&mut self) -> &mut ValueService<V>;

    fn invoke_service(&self) -> &InvokeService<S, V>;

    fn invoke_service_mut(&mut self) -> &mut InvokeService<S, V>;

    fn check_service(&self) -> &CheckService<S, V>;

    fn check_service_mut(&mut self) -> &mut CheckService<S, V>;

    fn get_data<T>(&self) -> Option<&T>
    where
        T: 'static;

    fn get_data_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static;

    fn ins_data<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static;

    fn rem_data<T>(&mut self) -> Option<T>
    where
        T: 'static;

    fn get_val(&self, uid: Uid) -> Option<&V>;

    fn get_vals(&self, uid: Uid) -> Option<&Vec<V>>;

    fn get_val_mut(&mut self, uid: Uid) -> Option<&mut V>;

    fn get_vals_mut(&mut self, uid: Uid) -> Option<&mut Vec<V>>;

    fn reg_callback<H, Args>(&mut self, uid: Uid, handler: H) -> &mut Self
    where
        Args: ExtractFromCtx<S, Error = Error> + 'static,
        H: Handler<S, Args, Output = Option<V>, Error = Error> + 'static;
}

impl<S, V> AServiceExt<S, V> for Services
where
    S: Set + 'static,
    V: From<Str> + 'static,
    S::Opt: Opt,
{
    fn new_default() -> Self {
        Services::new()
            .with(ValueService::<V>::new())
            .with(DataService::new())
            .with(InvokeService::<S, V>::new())
            .with(CheckService::<S, V>::new())
    }

    fn noa_service(&self) -> &NOAService {
        self.get::<NOAService>().unwrap()
    }

    fn noa_service_mut(&mut self) -> &mut NOAService {
        self.get_mut::<NOAService>().unwrap()
    }

    fn data_service(&self) -> &DataService {
        self.get::<DataService>().unwrap()
    }

    fn data_service_mut(&mut self) -> &mut DataService {
        self.get_mut::<DataService>().unwrap()
    }

    fn value_service(&self) -> &ValueService<V>
    where
        V: From<Str> + 'static,
    {
        self.get::<ValueService<V>>().unwrap()
    }

    fn value_service_mut(&mut self) -> &mut ValueService<V>
    where
        V: From<Str> + 'static,
    {
        self.get_mut::<ValueService<V>>().unwrap()
    }

    fn invoke_service(&self) -> &InvokeService<S, V>
    where
        S: Set + 'static,
        V: From<Str> + 'static,
        S::Opt: Opt,
    {
        self.get::<InvokeService<S, V>>().unwrap()
    }

    fn invoke_service_mut(&mut self) -> &mut InvokeService<S, V>
    where
        S: Set + 'static,
        V: From<Str> + 'static,
        S::Opt: Opt,
    {
        self.get_mut::<InvokeService<S, V>>().unwrap()
    }

    fn check_service(&self) -> &CheckService<S, V>
    where
        S: Set + 'static,
        V: From<Str> + 'static,
        S::Opt: Opt,
    {
        self.get::<CheckService<S, V>>().unwrap()
    }

    fn check_service_mut(&mut self) -> &mut CheckService<S, V>
    where
        S: Set + 'static,
        V: From<Str> + 'static,
        S::Opt: Opt,
    {
        self.get_mut::<CheckService<S, V>>().unwrap()
    }

    fn get_data<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.get::<DataService>().and_then(|v| v.get::<T>())
    }

    fn get_data_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.get_mut::<DataService>().and_then(|v| v.get_mut::<T>())
    }

    fn ins_data<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static,
    {
        self.get_mut::<DataService>().and_then(|v| v.insert(value))
    }

    fn rem_data<T>(&mut self) -> Option<T>
    where
        T: 'static,
    {
        self.get_mut::<DataService>().and_then(|v| v.remove::<T>())
    }

    fn get_val(&self, uid: Uid) -> Option<&V> {
        self.get::<ValueService<V>>().and_then(|v| v.val(uid))
    }

    fn get_vals(&self, uid: Uid) -> Option<&Vec<V>> {
        self.get::<ValueService<V>>().and_then(|v| v.vals(uid))
    }

    fn get_val_mut(&mut self, uid: Uid) -> Option<&mut V> {
        self.get_mut::<ValueService<V>>()
            .and_then(|v| v.val_mut(uid))
    }

    fn get_vals_mut(&mut self, uid: Uid) -> Option<&mut Vec<V>> {
        self.get_mut::<ValueService<V>>()
            .and_then(|v| v.vals_mut(uid))
    }

    fn reg_callback<H, Args>(&mut self, uid: Uid, handler: H) -> &mut Self
    where
        Args: ExtractFromCtx<S, Error = Error> + 'static,
        H: Handler<S, Args, Output = Option<V>, Error = Error> + 'static,
    {
        if let Some(v) = self.get_mut::<InvokeService<S, V>>() {
            v.register(uid, handler);
        } else {
            panic!(
                "Can not get InvokeServices from Services, pls check the callback return value! "
            )
        }
        self
    }
}
