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
use crate::ext::ServicesRawValExt;
use crate::ext::ServicesUsrValExt;
use crate::ext::ServicesValExt;
use crate::prelude::CheckService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::UsrValService;
use crate::ser::ValService;
use crate::set::Set;
use crate::Arc;
use crate::Error;
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

    fn ser_invoke<S: 'static>(&self) -> Result<&InvokeService<S>, Error> {
        self.service::<InvokeService<S>>()
    }

    fn ser_invoke_mut<S: 'static>(&mut self) -> Result<&mut InvokeService<S>, Error> {
        self.service_mut::<InvokeService<S>>()
    }

    fn ser_rawval<T: 'static>(&self) -> Result<&RawValService<T>, Error> {
        self.service::<RawValService<T>>()
    }

    fn ser_rawval_mut<T: 'static>(&mut self) -> Result<&mut RawValService<T>, Error> {
        self.service_mut::<RawValService<T>>()
    }

    fn ser_check<S: 'static>(&self) -> Result<&crate::prelude::CheckService<S>, Error> {
        self.service::<CheckService<S>>()
    }
}

impl ServicesRawValExt<crate::RawVal> for crate::RawVal {
    fn raw_val(uid: Uid, ser: &Services) -> Result<&crate::RawVal, Error> {
        ser.ser_rawval()?.val(uid)
    }

    fn raw_val_mut(uid: Uid, ser: &mut Services) -> Result<&mut crate::RawVal, Error> {
        ser.ser_rawval_mut()?.val_mut(uid)
    }

    fn raw_vals(uid: Uid, ser: &Services) -> Result<&Vec<crate::RawVal>, Error> {
        ser.ser_rawval()?.vals(uid)
    }

    fn raw_vals_mut(uid: Uid, ser: &mut Services) -> Result<&mut Vec<crate::RawVal>, Error> {
        ser.ser_rawval_mut()?.vals_mut(uid)
    }
}

impl<T> ServicesValExt<T> for T
where
    T: 'static,
{
    fn val(uid: Uid, ser: &Services) -> Result<&T, Error> {
        ser.ser_val()?.val(uid)
    }

    fn val_mut(uid: Uid, ser: &mut Services) -> Result<&mut T, Error> {
        ser.ser_val_mut()?.val_mut(uid)
    }

    fn vals(uid: Uid, ser: &Services) -> Result<&Vec<T>, Error> {
        ser.ser_val()?.vals(uid)
    }

    fn vals_mut(uid: Uid, ser: &mut Services) -> Result<&mut Vec<T>, Error> {
        ser.ser_val_mut()?.vals_mut(uid)
    }

    fn apply_filter<F: FnMut(&T) -> bool>(
        uid: Uid,
        ser: &mut Services,
        mut f: F,
    ) -> Result<Vec<T>, Error> {
        let vals = T::vals_mut(uid, ser)?;
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
}

impl<T> ServicesUsrValExt<T> for T
where
    T: 'static,
{
    fn usr_val(ser: &Services) -> Result<&T, Error> {
        ser.ser_usrval()?.val::<T>()
    }

    fn usr_val_mut(ser: &mut Services) -> Result<&mut T, Error> {
        ser.ser_usrval_mut()?.val_mut::<T>()
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
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// ser::Value::<PosList>::usr_val(&ser)?.test_pos(
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

impl<T: 'static> Value<T> {
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

impl<T: 'static, S: Set> Extract<S> for Value<T> {
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
