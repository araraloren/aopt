use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
use crate::ser::DataService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::ValService;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Uid;

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
    fn from(Arc: Arc<T>) -> Self {
        Data(Arc)
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
        f.debug_tuple("Data").field(&self.0).finish()
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

/// Simple wrapper of option value stored in [`RawValService`](crate::ser::RawValService).
/// It will clone the value from [`RawValService`](crate::ser::RawValService)
impl<S: Set> ExtractCtx<S> for RawVal {
    type Error = Error;

    fn extract(uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ser.service::<RawValService<RawVal>>()?.val(uid)?.clone())
    }
}

impl<S: Set> ExtractCtx<S> for Vec<RawVal> {
    type Error = Error;

    fn extract(uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ser.service::<RawValService<RawVal>>()?.vals(uid)?.clone())
    }
}
