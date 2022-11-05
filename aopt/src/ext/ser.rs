use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
use crate::ext::ServicesExt;
use crate::ser::DataService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
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

    fn ser_data(&self) -> Result<&DataService, Error> {
        self.service::<DataService>()
    }

    fn ser_data_mut(&mut self) -> Result<&mut DataService, Error> {
        self.service_mut::<DataService>()
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
}

/// Simple data wrapper of user data stored in [`DataService`](crate::ser::DataService).
///
/// UserData internally use [Arc](crate::Arc), it is cheap to clone.
pub struct Data<T: ?Sized>(Arc<T>);

impl<T> Data<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T: ?Sized> Data<T> {
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

/// UserData internally use Arc.
impl<T: ?Sized> Clone for Data<T> {
    fn clone(&self) -> Data<T> {
        Data(Arc::clone(&self.0))
    }
}

impl<T: ?Sized> From<Arc<T>> for Data<T> {
    fn from(val: Arc<T>) -> Self {
        Data(val)
    }
}

impl<T: 'static, S: Set> ExtractCtx<S> for Data<T> {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ser.service::<DataService>()?.data::<Data<T>>()?.clone())
    }
}

impl<T> Debug for Data<T>
where
    T: Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Data").field(&self.0).finish()
    }
}

impl<T: Display> Display for Data<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Data({})", self.0)
    }
}

impl<T> Deref for Data<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Data<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Serialize for Data<T>
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

impl<'de, T> Deserialize<'de> for Data<T>
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

/// Simple wrapper of option value stored in [`ValueService`](crate::ser::ValueService).
/// It will clone the value from [`ValueService`](crate::ser::ValueService)
pub struct Value<T>(T);

impl<T> Value<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Clone for Value<T>
where
    T: Clone,
{
    fn clone(&self) -> Value<T> {
        Value(self.0.clone())
    }
}

impl<T: Clone + 'static, S: Set> ExtractCtx<S> for Value<T> {
    type Error = Error;

    fn extract(uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Self(ser.service::<ValService>()?.val::<T>(uid)?.clone()))
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
    type Target = T;

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
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer)?))
    }
}

/// Simple wrapper of option value stored in [`ValueService`](crate::ser::ValueService).
/// It will clone the value from [`ValueService`](crate::ser::ValueService)
pub struct Values<T>(Vec<T>);

impl<T> Values<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self(values)
    }
}

impl<T> Clone for Values<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone + 'static, S: Set> ExtractCtx<S> for Values<T> {
    type Error = Error;

    fn extract(uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Self(ser.service::<ValService>()?.vals::<T>(uid)?.clone()))
    }
}

impl<T> Debug for Values<T>
where
    T: Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Values").field(&self.0).finish()
    }
}

impl<T> Display for Values<T>
where
    Vec<T>: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Values({})", self.0)
    }
}

impl<T> Deref for Values<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Values<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Serialize for Values<T>
where
    Vec<T>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Values<T>
where
    Vec<T>: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(<Vec<T>>::deserialize(deserializer)?))
    }
}

/// Simple wrapper of option value stored in [`RawValService`](crate::ser::RawValService).
/// It will clone the value from [`RawValService`](crate::ser::RawValService)
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawVal(crate::RawVal);

impl Deref for RawVal {
    type Target = crate::RawVal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawVal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for RawVal {
    type Error = Error;

    fn extract(uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Self(
            ser.service::<RawValService<crate::RawVal>>()?
                .val(uid)?
                .clone(),
        ))
    }
}

/// Simple wrapper of option value stored in [`RawValService`](crate::ser::RawValService).
/// It will clone the value from [`RawValService`](crate::ser::RawValService)
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawVals(Vec<crate::RawVal>);

impl Deref for RawVals {
    type Target = Vec<crate::RawVal>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawVals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for RawVals {
    type Error = Error;

    fn extract(uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Self(
            ser.service::<RawValService<crate::RawVal>>()?
                .vals(uid)?
                .clone(),
        ))
    }
}
