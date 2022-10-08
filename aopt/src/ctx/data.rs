use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
use crate::ser::DataService;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::set::Set;
use crate::Error;
use crate::Uid;

/// Simple data wrapper of user data stored in [`DataService`](crate::ser::DataService).
///
/// UserData internally use Rc, it is cheap to clone.
pub struct Data<T: ?Sized>(Rc<T>);

impl<T> Data<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ?Sized> Data<T> {
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Rc<T> {
        self.0
    }
}

/// UserData internally use Rc.
impl<T: ?Sized> Clone for Data<T> {
    fn clone(&self) -> Data<T> {
        Data(Rc::clone(&self.0))
    }
}

impl<T: ?Sized> From<Rc<T>> for Data<T> {
    fn from(rc: Rc<T>) -> Self {
        Data(rc)
    }
}

impl<T: 'static, S: Set> ExtractCtx<S> for Data<T> {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ser
            .ser::<DataService>()?
            .get::<Data<T>>()
            .ok_or_else(|| Error::raise_error("Can not get data from data service"))?
            .clone())
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
    type Target = Rc<T>;

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
    Rc<T>: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(Rc::<T>::deserialize(deserializer)?))
    }
}
