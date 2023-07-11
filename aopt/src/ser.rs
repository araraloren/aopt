use std::any::type_name;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::map::AnyMap;
use crate::map::Entry;
use crate::map::ErasedTy;
use crate::raise_error;
use crate::Error;

/// Some convenient function access the [`AppServices`](crate::ser::AppServices).
pub trait ServicesExt {
    fn ser_app(&self) -> &AppServices;

    fn ser_app_mut(&mut self) -> &mut AppServices;
}

pub trait ServicesValExt {
    /// Get the user value reference of option `uid` from [`AppServices`].
    fn sve_insert<T: ErasedTy>(&mut self, val: T) -> Option<T>;

    /// Get the user value reference of option `uid` from [`AppServices`].
    fn sve_val<T: ErasedTy>(&self) -> Result<&T, Error>;

    /// Get the user value mutable reference of option `uid` from [`AppServices`].
    fn sve_val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error>;

    /// Take the user value of option `uid` from [`AppServices`].
    fn sve_take_val<T: ErasedTy>(&mut self) -> Result<T, Error>;
}

/// A service can keep any type data, user can get the data inside [`hanlder`](crate::ctx::InvokeHandler) of option.
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// #
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #[derive(Debug, PartialEq)]
/// struct MyVec(pub Vec<i32>);
///
/// let mut services = AppServices::new();
///
/// services.sve_insert(MyVec(vec![42]));
/// services.sve_insert(42i64);
///
/// /// get value of MyVec from AppServices
/// assert_eq!(services.sve_val::<MyVec>()?.0[0], 42);
/// /// modfify the value
/// services.sve_val_mut::<MyVec>()?.0.push(18);
/// /// check the value of MyVec
/// assert_eq!(services.sve_val::<MyVec>()?.0[1], 18);
///
/// assert_eq!(services.sve_val::<i64>()?, &42);
/// #
/// #    Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct AppServices(UsrValService);

impl AppServices {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl ServicesExt for AppServices {
    fn ser_app(&self) -> &AppServices {
        self
    }

    fn ser_app_mut(&mut self) -> &mut AppServices {
        self
    }
}

impl ServicesValExt for AppServices {
    fn sve_insert<T: ErasedTy>(&mut self, val: T) -> Option<T> {
        self.0.insert(val)
    }

    fn sve_val<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.0.val::<T>()
    }

    fn sve_val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.0.val_mut::<T>()
    }

    fn sve_take_val<T: ErasedTy>(&mut self) -> Result<T, Error> {
        self.0.remove::<T>().ok_or_else(|| {
            raise_error!(
                "Can not take value type `{}` from AppServices",
                type_name::<T>()
            )
        })
    }
}

impl Deref for AppServices {
    type Target = UsrValService;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AppServices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// [`UsrValService`] can save values of any type.
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;   
/// #
/// # fn main() -> Result<(), Error> {
/// let mut service = UsrValService::new();
///
/// assert_eq!(service.contain_type::<Vec<i32>>(), false);
/// assert_eq!(service.insert(vec![42]), None);
/// assert_eq!(service.contain_type::<Vec<i32>>(), true);
///
/// assert_eq!(service.val::<Vec<i32>>()?, &vec![42]);
/// service.val_mut::<Vec<i32>>()?.push(256);
/// assert_eq!(service.val::<Vec<i32>>()?, &vec![42, 256]);
///
/// assert_eq!(service.val_mut::<Vec<i32>>()?, &mut vec![42, 256]);
/// assert_eq!(service.val_mut::<Vec<i32>>()?.pop(), Some(256));
/// assert_eq!(service.val::<Vec<i32>>()?, &vec![42]);
///
/// service.entry::<Vec<u64>>().or_insert(vec![9, 0, 2, 5]);
/// assert_eq!(service.entry::<Vec<u64>>().or_default().pop(), Some(5));
///
/// service.val_mut::<Vec<i32>>()?.pop();
/// assert_eq!(service.val_mut::<Vec<i32>>()?.len(), 0);
///
/// assert_eq!(service.remove::<Vec<u64>>(), Some(vec![9, 0, 2]));
/// assert_eq!(service.contain_type::<u64>(), false);
/// assert_eq!(service.get::<Vec<u64>>(), None);
/// assert_eq!(service.get_mut::<Vec<i32>>(), Some(&mut vec![]));
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct UsrValService(AnyMap);

impl Debug for UsrValService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("UsrValService").field(&self.0).finish()
    }
}

impl UsrValService {
    pub fn new() -> Self {
        Self(AnyMap::default())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contain_type<T: ErasedTy>(&self) -> bool {
        self.0.contain::<T>()
    }

    pub fn insert<T: ErasedTy>(&mut self, value: T) -> Option<T> {
        self.0.insert(value)
    }

    pub fn remove<T: ErasedTy>(&mut self) -> Option<T> {
        self.0.remove::<T>()
    }

    pub fn get<T: ErasedTy>(&self) -> Option<&T> {
        self.0.value::<T>()
    }

    pub fn get_mut<T: ErasedTy>(&mut self) -> Option<&mut T> {
        self.0.value_mut::<T>()
    }

    pub fn val<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.get::<T>().ok_or_else(|| {
            raise_error!(
                "Can not find reference for type `{:?}` in UsrValService",
                type_name::<T>()
            )
        })
    }

    pub fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.get_mut::<T>().ok_or_else(|| {
            raise_error!(
                "Can not find mutable reference for type `{:?}` in UsrValService",
                type_name::<T>()
            )
        })
    }

    pub fn entry<T: ErasedTy>(&mut self) -> Entry<'_, T> {
        self.0.entry::<T>()
    }
}
