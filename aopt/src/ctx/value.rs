use serde::Serialize;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
use crate::prelude::ValueService;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::ser::ValueServiceExt;
use crate::set::Set;
use crate::Error;
use crate::Uid;

/// Simple wrapper of option value stored in [`ValueService`](crate::ser::ValueService).
///
/// UserData internally use Rc, it is cheap to clone.
pub struct Value<T>(T);

impl<T> Value<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

/// UserData internally use Rc.
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
        Ok(Self(ser.ser::<ValueService<T>>()?.val(uid)?.clone()))
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
