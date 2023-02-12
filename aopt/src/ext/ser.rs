//! The structs hold the data collect from [`AppServices`](crate::ser::AppServices).
//! They are all implemented [`Extract`].
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::map::ErasedTy;
use crate::ser::ServicesValExt;
use crate::ARef;
use crate::Error;

#[cfg(feature = "sync")]
pub struct Value<T: ?Sized>(ARef<T>);

#[cfg(not(feature = "sync"))]
/// Simple wrapper of user value saved in [`UsrValService`](crate::ser::UsrValService).
///
/// Value internally use [ARef](crate::ARef), it is cheap to clone.
/// Before used it in `handler` which register in [`Invoker`](crate::ctx::Invoker),
/// you need add it to [`UsrValService`](crate::ser::UsrValService).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
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
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// ser.sve_insert(ser::Value::new(PosList(RefCell::new(vec![]))));
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("pos_v=p@*")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, disable: ctx::Value<bool>| {
///         assert_eq!(&true, disable.deref());
///         Ok(Some(false))
///     });
///
/// inv.entry(1)
///     .on(
///         |_: &mut ASet, _: &mut ASer, raw_val: ctx::RawVal, data: ser::Value<PosList>| {
///             data.add_pos(raw_val.clone_rawval());
///             Ok(Some(true))
///         },
///     );
///
/// let args = Args::from_array(["app", "--/bool", "set", "42", "foo", "bar"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set[0].val::<bool>()?, &false);
/// ser.sve_val::<ser::Value::<PosList>>()?.test_pos(
///     ["set", "42", "foo", "bar"]
///         .into_iter()
///         .map(RawVal::from)
///         .collect(),
/// );
/// # Ok(())
/// # }
/// ```
pub struct Value<T: ?Sized>(ARef<T>);

impl<T> Value<T> {
    pub fn new(value: T) -> Self {
        Self(ARef::new(value))
    }
}

impl<T: ErasedTy> Value<T> {
    pub fn extract_ser(ser: &impl ServicesValExt) -> Result<Self, Error> {
        Ok(ser
            .sve_val::<Value<T>>()
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

    pub fn into_inner(self) -> ARef<T> {
        self.0
    }
}

/// Value internally use ARef.
impl<T: ?Sized> Clone for Value<T> {
    fn clone(&self) -> Value<T> {
        Value(ARef::clone(&self.0))
    }
}

impl<T: ?Sized> From<ARef<T>> for Value<T> {
    fn from(val: ARef<T>) -> Self {
        Value(val)
    }
}

impl<T: ErasedTy, Set, Ser: ServicesValExt> Extract<Set, Ser> for Value<T> {
    type Error = Error;

    fn extract(_set: &Set, ser: &Ser, _ctx: &Ctx) -> Result<Self, Self::Error> {
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
    type Target = ARef<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Value<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for Value<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Value<T>
where
    ARef<T>: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(ARef::<T>::deserialize(deserializer)?))
    }
}
