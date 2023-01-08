pub(crate) mod anyval;
pub(crate) mod rawval;
pub(crate) mod userval;

pub use self::anyval::AnyValEntry;
pub use self::anyval::AnyValService;
pub use self::rawval::RawValService;
pub use self::userval::UsrValService;

use crate::map::ErasedTy;
use crate::Error;
use crate::RawVal;
use crate::Uid;

/// Services keep different type [`Service`]s in a map.
///
/// # Examples
/// ```rust
/// # use aopt::Result;
/// # use aopt::prelude::*;
/// # use aopt::astr;
/// # use aopt::Str;
/// #
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
///     assert_eq!(services.service::<MyVec>()?.0[0], 42);
///     // modfify the service value
///     services.service_mut::<MyVec>()?.0.push(18);
///     // check the value of MyVec
///     assert_eq!(services.service::<MyVec>()?.0[1], 18);
///
///     // register a new service
///     services.register(I32(42));
///     assert!(services.contain::<I32>());
///
///     // unregister service from
///     services.remove::<MyVec>();
///     assert!(!services.contain::<MyVec>());
///
///     Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct Services {
    any: AnyValService,

    usr: UsrValService,

    raw: RawValService<RawVal>,
}

/// Some convenient function access the [`Service`](crate::ser::Service) in [`Services`].
pub trait ServicesExt {
    /// Get [`ValService`] reference.
    fn ser_val(&self) -> &AnyValService;

    /// Get [`ValService`] mutable reference.
    fn ser_val_mut(&mut self) -> &mut AnyValService;

    /// Get [`UsrValService`] reference.
    fn ser_usrval(&self) -> &UsrValService;

    /// Get [`UsrValService`] mutable reference.
    fn ser_usrval_mut(&mut self) -> &mut UsrValService;

    /// Get [`RawValService`] reference.
    fn ser_rawval(&self) -> &RawValService<RawVal>;

    /// Get [`RawValService`] mutable reference.
    fn ser_rawval_mut(&mut self) -> &mut RawValService<RawVal>;
}

pub trait ServicesValExt {
    /// Get the last value reference of option `uid` from [`ValService`].
    fn sve_val<T: ErasedTy>(&self, uid: Uid) -> Result<&T, Error>;

    /// Get the last value mutable reference of option `uid` from [`ValService`].
    fn sve_val_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut T, Error>;

    /// Take last value of option `uid` from [`ValService`].
    fn sve_take_val<T: ErasedTy>(&mut self, uid: Uid) -> Result<T, Error>;

    /// Get the values reference of option `uid` from [`ValService`].
    fn sve_vals<T: ErasedTy>(&self, uid: Uid) -> Result<&Vec<T>, Error>;

    /// Get the values mutable reference of option `uid` from [`ValService`].
    fn sve_vals_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error>;

    /// Take the values of option `uid` from [`ValService`].
    fn sve_take_vals<T: ErasedTy>(&mut self, uid: Uid) -> Result<Vec<T>, Error>;

    /// Apply filter on the values of option from [`ValService`].
    /// The `F` should return true if you want remove the element.
    fn sve_filter<T: ErasedTy>(
        &mut self,
        uid: Uid,
        f: impl FnMut(&T) -> bool,
    ) -> Result<Vec<T>, Error>;

    /// Get the user value reference of option `uid` from [`UsrValService`].
    fn sve_usrval<T: ErasedTy>(&self) -> Result<&T, Error>;

    /// Get the user value mutable reference of option `uid` from [`UsrValService`].
    fn sve_usrval_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error>;

    /// Take the user value of option `uid` from [`UsrValService`].
    fn sve_take_usrval<T: ErasedTy>(&mut self) -> Result<T, Error>;

    /// Get the raw value reference of option `uid` from [`RawValService`].
    fn sve_rawval(&self, uid: Uid) -> Result<&RawVal, Error>;

    /// Get the raw value mutable reference of option `uid` from [`RawValService`].
    fn sve_rawval_mut(&mut self, uid: Uid) -> Result<&mut RawVal, Error>;

    /// Get the raw values reference of option `uid` from [`RawValService`].
    fn sve_rawvals(&self, uid: Uid) -> Result<&Vec<RawVal>, Error>;

    /// Get the raw values mutable reference of option `uid` from [`RawValService`].
    fn sve_rawvals_mut(&mut self, uid: Uid) -> Result<&mut Vec<RawVal>, Error>;
}

impl ServicesExt for Services {
    fn ser_val(&self) -> &AnyValService {
        &self.any
    }

    fn ser_val_mut(&mut self) -> &mut AnyValService {
        &mut self.any
    }

    fn ser_usrval(&self) -> &UsrValService {
        &self.usr
    }

    fn ser_usrval_mut(&mut self) -> &mut UsrValService {
        &mut self.usr
    }

    fn ser_rawval(&self) -> &RawValService<RawVal> {
        &self.raw
    }

    fn ser_rawval_mut(&mut self) -> &mut RawValService<RawVal> {
        &mut self.raw
    }
}

impl ServicesValExt for Services {
    fn sve_val<T: ErasedTy>(&self, uid: Uid) -> Result<&T, Error> {
        self.ser_val().val(uid)
    }

    fn sve_val_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut T, Error> {
        self.ser_val_mut().val_mut(uid)
    }

    fn sve_take_val<T: ErasedTy>(&mut self, uid: Uid) -> Result<T, Error> {
        self.ser_val_mut()
            .pop(uid)
            .ok_or_else(|| Error::raise_error("Can not take value from ValService"))
    }

    fn sve_vals<T: ErasedTy>(&self, uid: Uid) -> Result<&Vec<T>, Error> {
        self.ser_val().vals(uid)
    }

    fn sve_vals_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error> {
        self.ser_val_mut().vals_mut(uid)
    }

    fn sve_take_vals<T: ErasedTy>(&mut self, uid: Uid) -> Result<Vec<T>, Error> {
        self.ser_val_mut()
            .remove(uid)
            .ok_or_else(|| Error::raise_error("Can not take values from ValService"))
    }

    fn sve_filter<T: ErasedTy>(
        &mut self,
        uid: Uid,
        mut f: impl FnMut(&T) -> bool,
    ) -> Result<Vec<T>, Error> {
        let vals = self.sve_vals_mut::<T>(uid)?;
        let mut i = 0;
        let mut removed = vec![];

        while i < vals.len() {
            if (f)(&vals[i]) {
                removed.push(vals.remove(i));
            } else {
                i += 1;
            }
        }
        Ok(removed)
    }

    fn sve_usrval<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.ser_usrval().val::<T>()
    }

    fn sve_usrval_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.ser_usrval_mut().val_mut::<T>()
    }

    fn sve_take_usrval<T: ErasedTy>(&mut self) -> Result<T, Error> {
        self.ser_usrval_mut()
            .remove::<T>()
            .ok_or_else(|| Error::raise_error("Can not take value from UsrValService"))
    }

    /// Get the raw value reference of option `uid` from [`RawValService`].
    fn sve_rawval(&self, uid: Uid) -> Result<&RawVal, Error> {
        self.ser_rawval().val(uid)
    }

    /// Get the raw value mutable reference of option `uid` from [`RawValService`].
    fn sve_rawval_mut(&mut self, uid: Uid) -> Result<&mut RawVal, Error> {
        self.ser_rawval_mut().val_mut(uid)
    }

    /// Get the raw values reference of option `uid` from [`RawValService`].
    fn sve_rawvals(&self, uid: Uid) -> Result<&Vec<RawVal>, Error> {
        self.ser_rawval().vals(uid)
    }

    /// Get the raw values mutable reference of option `uid` from [`RawValService`].
    fn sve_rawvals_mut(&mut self, uid: Uid) -> Result<&mut Vec<RawVal>, Error> {
        self.ser_rawval_mut().vals_mut(uid)
    }
}
