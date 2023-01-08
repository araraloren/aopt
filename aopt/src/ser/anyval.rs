use std::fmt::Debug;
use std::marker::PhantomData;

use crate::map::AnyMap;
use crate::map::ErasedTy;
use crate::Error;
use crate::{HashMap, Uid};

/// Keep any type value in [`HashMap`] with key [`Uid`].
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct MyData;
///
/// let mut vs = ValService::default();
///
/// vs.push(0, 42i64);
/// vs.push(0, 36i64);
/// vs.push(1, 28u64);
/// vs.push(1, 14u64);
/// vs.push(2, MyData {});
/// vs.push(2, 3.14f64);
///
/// assert_eq!(vs.val::<i64>(0)?, &36i64);
/// assert_eq!(vs.vals::<i64>(0)?, &vec![42, 36]);
/// assert_eq!(vs.contain_type::<i64>(0), true);
/// assert_eq!(vs.contain_type::<i32>(0), false);
///
/// assert_eq!(vs.val::<u64>(1)?, &14u64);
/// assert_eq!(vs.vals::<u64>(1)?, &vec![28, 14]);
/// assert_eq!(vs.contain_type::<u64>(1), true);
/// assert_eq!(vs.contain_type::<f64>(1), false);
///
/// assert_eq!(vs.val::<MyData>(2)?, &MyData {});
/// assert_eq!(vs.val::<f64>(2)?, &3.14f64);
/// assert_eq!(vs.contain_type::<MyData>(2), true);
/// assert_eq!(vs.contain_type::<f64>(2), true);
/// #
/// #    Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct AnyValService {
    inner: HashMap<Uid, AnyMap>,
}

impl Debug for AnyValService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyValService")
            .field("inner", &self.inner)
            .finish()
    }
}

impl AnyValService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contain(&self, uid: Uid) -> bool {
        self.inner.contains_key(&uid)
    }

    pub fn contain_type<T: ErasedTy>(&self, uid: Uid) -> bool {
        self.inner
            .get(&uid)
            .map(|v| v.contain::<Vec<T>>())
            .unwrap_or_default()
    }

    pub fn get<T: ErasedTy>(&self, uid: Uid) -> Option<&T> {
        self.gets::<T>(uid).and_then(|v| v.last())
    }

    pub fn get_mut<T: ErasedTy>(&mut self, uid: Uid) -> Option<&mut T> {
        self.gets_mut::<T>(uid).and_then(|v| v.last_mut())
    }

    pub fn gets<T: ErasedTy>(&self, uid: Uid) -> Option<&Vec<T>> {
        self.inner.get(&uid).and_then(|map| map.get::<Vec<T>>())
    }

    pub fn gets_mut<T: ErasedTy>(&mut self, uid: Uid) -> Option<&mut Vec<T>> {
        self.inner
            .get_mut(&uid)
            .and_then(|map| map.get_mut::<Vec<T>>())
    }

    pub fn push<T: ErasedTy>(&mut self, uid: Uid, val: T) -> &mut Self {
        self.inner
            .entry(uid)
            .or_default()
            .entry::<Vec<T>>()
            .or_insert_with(Vec::new)
            .push(val);
        self
    }

    pub fn pop<T: ErasedTy>(&mut self, uid: Uid) -> Option<T> {
        self.inner
            .get_mut(&uid)
            .and_then(|v| v.get_mut::<Vec<T>>())
            .and_then(|v| v.pop())
    }

    pub fn set<T: ErasedTy>(&mut self, uid: Uid, vals: Vec<T>) -> Option<Vec<T>> {
        self.inner.entry(uid).or_default().insert(vals)
    }

    pub fn remove<T: ErasedTy>(&mut self, uid: Uid) -> Option<Vec<T>> {
        self.inner.get_mut(&uid).and_then(|v| v.remove::<Vec<T>>())
    }

    pub fn entry<T>(&mut self, uid: Uid) -> AnyValEntry<'_, Vec<T>> {
        AnyValEntry::new(uid, self.inner.entry(uid).or_default())
    }

    pub fn val<T: ErasedTy>(&self, uid: Uid) -> Result<&T, Error> {
        self.get(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    pub fn val_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut T, Error> {
        self.get_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    pub fn vals<T: ErasedTy>(&self, uid: Uid) -> Result<&Vec<T>, Error> {
        self.gets(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    pub fn vals_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error> {
        self.gets_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }
}

pub struct AnyValEntry<'a, T> {
    uid: Uid,

    map: &'a mut AnyMap,

    marker: PhantomData<T>,
}

impl<'a, T> AnyValEntry<'a, T> {
    pub fn new(uid: Uid, map: &'a mut AnyMap) -> Self {
        Self {
            uid,
            map,
            marker: PhantomData::default(),
        }
    }
}

impl<'a, T> AnyValEntry<'a, T>
where
    T: ErasedTy,
{
    pub fn key(&self) -> Uid {
        self.uid
    }

    pub fn or_insert(self, val: T) -> &'a mut T {
        self.map.entry().or_insert(val)
    }

    pub fn or_insert_with<F>(self, f: F) -> &'a mut T
    where
        F: FnOnce() -> T,
    {
        self.map.entry::<T>().or_insert(f())
    }

    pub fn or_insert_with_key<F>(self, f: F) -> &'a mut T
    where
        F: FnOnce(&Uid) -> T,
    {
        let val = f(&self.key());
        self.map.entry::<T>().or_insert(val)
    }

    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut T),
    {
        self.map.entry::<T>().and_modify(|v| f(v));
        self
    }
}
