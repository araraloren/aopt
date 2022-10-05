use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

use crate::ctx::Context;
use crate::ctx::ExtractFromCtx;
use crate::prelude::DataService;
use crate::prelude::ServicesExt;
use crate::ser::Services;
use crate::Error;
use crate::SimpleSet;
use crate::Uid;

/// Simple data wrapper of user data stored in [`DataService`](crate::ser::DataService).
///
/// UserData internally use Rc, it is cheap to clone.
pub struct UserData<T: ?Sized>(Rc<T>);

impl<T> UserData<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ?Sized> UserData<T> {
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    pub fn into_inner(self) -> Rc<T> {
        self.0
    }
}

/// UserData internally use Rc.
impl<T: ?Sized> Clone for UserData<T> {
    fn clone(&self) -> UserData<T> {
        UserData(Rc::clone(&self.0))
    }
}

impl<T: ?Sized> From<Rc<T>> for UserData<T> {
    fn from(rc: Rc<T>) -> Self {
        UserData(rc)
    }
}

impl<T> ExtractFromCtx<SimpleSet> for UserData<T>
where
    T: 'static,
{
    type Error = Error;

    fn extract_from(
        _uid: Uid,
        _set: &SimpleSet,
        ser: &mut Services,
        _ctx: Context,
    ) -> Result<Self, Self::Error> {
        Ok(ser
            .get_service::<DataService>()?
            .get::<UserData<T>>()
            .ok_or_else(|| Error::raise_error("Can not get data from data service"))?
            .clone())
    }
}

impl<T> Debug for UserData<T>
where
    T: Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Data").field(&self.0).finish()
    }
}

impl<T> Deref for UserData<T> {
    type Target = Rc<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for UserData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
