//! The structs hold the data collect from [`Services`](crate::ser::Services).
//! They are all implemented [`Extract`].
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::ext::ServicesExt;
use crate::ext::ServicesValExt;
use crate::map::ErasedTy;
use crate::prelude::CheckService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::UsrValService;
use crate::ser::ValService;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Uid;

impl ServicesExt for Services {
    fn ser_val(&self) -> Result<&ValService, Error> {
        self.service::<ValService>()
    }

    fn ser_val_mut(&mut self) -> Result<&mut ValService, Error> {
        self.service_mut::<ValService>()
    }

    fn ser_usrval(&self) -> Result<&UsrValService, Error> {
        self.service::<UsrValService>()
    }

    fn ser_usrval_mut(&mut self) -> Result<&mut UsrValService, Error> {
        self.service_mut::<UsrValService>()
    }

    fn ser_invoke<S: Set + 'static>(&self) -> Result<&InvokeService<S>, Error> {
        self.service::<InvokeService<S>>()
    }

    fn ser_invoke_mut<S: Set + 'static>(&mut self) -> Result<&mut InvokeService<S>, Error> {
        self.service_mut::<InvokeService<S>>()
    }

    fn ser_rawval<T: ErasedTy>(&self) -> Result<&RawValService<T>, Error> {
        self.service::<RawValService<T>>()
    }

    fn ser_rawval_mut<T: ErasedTy>(&mut self) -> Result<&mut RawValService<T>, Error> {
        self.service_mut::<RawValService<T>>()
    }

    fn ser_check<S: Set + 'static>(&self) -> Result<&CheckService<S>, Error> {
        self.service::<CheckService<S>>()
    }
}

impl ServicesValExt for Services {
    fn sve_val<T: ErasedTy>(&self, uid: Uid) -> Result<&T, Error> {
        self.ser_val()?.val(uid)
    }

    fn sve_val_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut T, Error> {
        self.ser_val_mut()?.val_mut(uid)
    }

    fn sve_take_val<T: ErasedTy>(&mut self, uid: Uid) -> Result<T, Error> {
        self.ser_val_mut()?
            .pop(uid)
            .ok_or_else(|| Error::raise_error("Can not take value from ValService"))
    }

    fn sve_vals<T: ErasedTy>(&self, uid: Uid) -> Result<&Vec<T>, Error> {
        self.ser_val()?.vals(uid)
    }

    fn sve_vals_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error> {
        self.ser_val_mut()?.vals_mut(uid)
    }

    fn sve_take_vals<T: ErasedTy>(&mut self, uid: Uid) -> Result<Vec<T>, Error> {
        self.ser_val_mut()?
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
        self.ser_usrval()?.val::<T>()
    }

    fn sve_usrval_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.ser_usrval_mut()?.val_mut::<T>()
    }

    fn sve_take_usrval<T: ErasedTy>(&mut self) -> Result<T, Error> {
        self.ser_usrval_mut()?
            .remove::<T>()
            .ok_or_else(|| Error::raise_error("Can not take value from UsrValService"))
    }

    /// Get the raw value reference of option `uid` from [`RawValService`].
    fn sve_rawval(&self, uid: Uid) -> Result<&RawVal, Error> {
        self.ser_rawval()?.val(uid)
    }

    /// Get the raw value mutable reference of option `uid` from [`RawValService`].
    fn sve_rawval_mut(&mut self, uid: Uid) -> Result<&mut RawVal, Error> {
        self.ser_rawval_mut()?.val_mut(uid)
    }

    /// Get the raw values reference of option `uid` from [`RawValService`].
    fn sve_rawvals(&self, uid: Uid) -> Result<&Vec<RawVal>, Error> {
        self.ser_rawval()?.vals(uid)
    }

    /// Get the raw values mutable reference of option `uid` from [`RawValService`].
    fn sve_rawvals_mut(&mut self, uid: Uid) -> Result<&mut Vec<RawVal>, Error> {
        self.ser_rawval_mut()?.vals_mut(uid)
    }
}

/// Simple wrapper of user value stored in [`UsrValService`](crate::ser::UsrValService).
///
/// Value internally use [Arc](crate::Arc), it is cheap to clone.
/// Before used it in `handler` which register in [`InvokeService`](crate::ser::InvokeService),
/// you need add it to [`UsrValService`].
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use aopt::RawVal;
/// # use std::cell::RefCell;
/// # use std::ops::Deref;
/// # fn main() -> Result<(), Error> {
/// #
/// #[derive(Debug, Clone)]
/// pub struct PosList(RefCell<Vec<RawVal>>);
///
/// impl PosList {
///     pub fn add_pos(&self, val: RawVal) {
///         self.0.borrow_mut().push(val);
///     }
///
///     pub fn test_pos(&self, test: Vec<RawVal>) {
///         assert_eq!(self.0.borrow().len(), test.len());
///         for (vall, valr) in self.0.borrow().iter().zip(test.iter()) {
///             assert_eq!(vall, valr);
///         }
///     }
/// }
///
///
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// ser.ser_usrval_mut()?
///     .insert(ser::Value::new(PosList(RefCell::new(vec![]))));
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("pos_v=p@*")?.run()?;
/// ser.ser_invoke_mut()?
///     .entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, disable: ctx::Disable| {
///         assert_eq!(&true, disable.deref());
///         Ok(Some(false))
///     });
///
/// ser.ser_invoke_mut()?
///     .entry(1)
///     .on(
///         |_: &mut ASet, _: &mut ASer, raw_val: ctx::RawVal, data: ser::Value<PosList>| {
///             data.add_pos(raw_val.clone_rawval());
///             Ok(Some(true))
///         },
///     );
///
/// let args = Args::new(["--/bool", "set", "42", "foo", "bar"].into_iter());
///
/// policy.parse(&mut set, &mut ser, Arc::new(args))?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// ser.sve_usrval::<ser::Value::<PosList>>()?.test_pos(
///     ["set", "42", "foo", "bar"]
///         .into_iter()
///         .map(RawVal::from)
///         .collect(),
/// );
/// # Ok(())
/// # }
/// ```
pub struct Value<T: ?Sized>(Arc<T>);

impl<T> Value<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T: ErasedTy> Value<T> {
    pub fn extract_ser(ser: &Services) -> Result<Self, Error> {
        Ok(ser
            .ser_usrval()
            .map_err(|e| {
                Error::sp_extract_error(format!("can not access UsrValServices: {:?}", e))
            })?
            .val::<Value<T>>()
            .map_err(|e| {
                Error::sp_extract_error(format!(
                    "can not get value of type {}: {:?}",
                    std::any::type_name::<Value<T>>(),
                    e
                ))
            })?
            .clone())
    }
}

impl<T: ?Sized> Value<T> {
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

/// Value internally use Arc.
impl<T: ?Sized> Clone for Value<T> {
    fn clone(&self) -> Value<T> {
        Value(Arc::clone(&self.0))
    }
}

impl<T: ?Sized> From<Arc<T>> for Value<T> {
    fn from(val: Arc<T>) -> Self {
        Value(val)
    }
}

impl<T: ErasedTy, S: Set> Extract<S> for Value<T> {
    type Error = Error;

    fn extract(_set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Self::extract_ser(ser)
    }
}

impl<T> Debug for Value<T>
where
    T: Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value").field(&self.0).finish()
    }
}

impl<T: Display> Display for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value({})", self.0)
    }
}

impl<T> Deref for Value<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Value<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Serialize for Value<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Value<T>
where
    Arc<T>: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(Arc::<T>::deserialize(deserializer)?))
    }
}
