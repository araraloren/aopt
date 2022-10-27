use serde::Serialize;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
use crate::prelude::RawValService;
use crate::ser::RawValServiceExt;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::set::Set;
use crate::Error;
use crate::Uid;

/// Simple wrapper of option value stored in [`ValueService`](crate::ser::ValueService).
///
/// UserData internally use Rc, it is cheap to clone.
pub struct RawVal<T>(T);

impl<T> RawVal<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

/// UserData internally use Rc.
impl<T> Clone for RawVal<T>
where
    T: Clone,
{
    fn clone(&self) -> RawVal<T> {
        RawVal(self.0.clone())
    }
}

impl<T: Clone + 'static, S: Set> ExtractCtx<S> for RawVal<T> {
    type Error = Error;

    fn extract(uid: Uid, _set: &S, ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Self(ser.ser::<RawValService<T>>()?.raw_val(uid)?.clone()))
    }
}

impl<T> Debug for RawVal<T>
where
    T: Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Data").field(&self.0).finish()
    }
}

impl<T> Deref for RawVal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RawVal<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Serialize for RawVal<T>
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