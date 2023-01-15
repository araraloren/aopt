use std::any::type_name;
use std::fmt::Debug;

use crate::map::AnyMap;
use crate::map::Entry;
use crate::map::ErasedTy;
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

/// Services manage the [`AnyValService`], [`RawValService`] and [`UsrValService`].
///
/// [`AnyValService`] is use for storing the parsed value of option.
/// [`RawValService`] is use for storing the raw value of option.
/// [`UsrValService`] is use for storing any type value can reference in option handler.
///
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::RawVal;
/// #
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///     #[derive(Debug, PartialEq)]
///     struct MyVec(pub Vec<i32>);
///
///     #[derive(Debug, PartialEq)]
///     struct I32(i32);
///
///     let mut services = Services::new();
///
///     services.ser_usrval_mut().insert(MyVec(vec![42]));
///
///     // get value from of UsrValService
///     assert_eq!(services.ser_usrval().val::<MyVec>()?.0[0], 42);
///     // modfify the value of UsrValServices
///     services.ser_usrval_mut().val_mut::<MyVec>()?.0.push(18);
///     // check the value of MyVec
///     assert_eq!(services.ser_usrval().val::<MyVec>()?.0[1], 18);
///
///     // push a new value to option 0
///     services.ser_val_mut().push(0, I32(42));
///     assert!(services.ser_val().contain(0));
///     assert!(services.ser_val().contain_type::<I32>(0));
///
///     // pop a new value from option 0
///     let ret: Option<I32> = services.ser_val_mut().pop(0);
///     assert!(ret.is_some());
///     assert_eq!(ret, Some(I32(42)));
///     // left empty vector in AnyValService
///     assert!(services.ser_val().contain(0));
///     assert!(services.ser_val().vals::<I32>(0)?.is_empty());
///
///     services.ser_rawval_mut().push(0, RawVal::from("value1"));
///     assert!(services.ser_rawval().contain(0));
///     assert_eq!(services.ser_rawval().val(0)?, &RawVal::from("value1"));
///
///     // pop a new value from option 0
///     let ret: Option<RawVal> = services.ser_rawval_mut().pop(0);
///     assert!(ret.is_some());
///     assert_eq!(ret, Some(RawVal::from("value1")));
///     // left empty vector in RawValService
///     assert!(services.ser_rawval().contain(0));
///     assert!(services.ser_rawval().vals(0)?.is_empty());
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
            Error::raise_error(format!(
                "Can not take value type {} from AppServices",
                type_name::<T>()
            ))
        })
    }
}

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

    pub fn contain<T: ErasedTy>(&self) -> bool {
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
            Error::raise_error(format!(
                "Can not find reference for type {{{:?}}} in UsrValService",
                type_name::<T>()
            ))
        })
    }

    pub fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.get_mut::<T>().ok_or_else(|| {
            Error::raise_error(format!(
                "Can not find mutable reference for type {{{:?}}} in UsrValService",
                type_name::<T>()
            ))
        })
    }

    pub fn entry<T: ErasedTy>(&mut self) -> Entry<'_, T> {
        self.0.entry::<T>()
    }
}
