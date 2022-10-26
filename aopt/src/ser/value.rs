use std::marker::PhantomData;

use super::Service;
use crate::map::Map;
use crate::Error;
use crate::{astr, HashMap, Uid};

#[derive(Default)]
pub struct ValService {
    inner: HashMap<Uid, Map>,
}

impl Service for ValService {
    fn service_name() -> crate::Str {
        astr("ValService")
    }
}

impl ValService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contain<T>(&self, uid: Uid) -> bool {
        self.inner.contains_key(&uid)
    }

    pub fn get<T>(&self, uid: Uid) -> Option<&T>
    where
        T: 'static,
    {
        self.inner.get(&uid).and_then(|map| map.get::<T>())
    }

    pub fn get_mut<T>(&mut self, uid: Uid) -> Option<&mut T>
    where
        T: 'static,
    {
        self.inner.get_mut(&uid).and_then(|map| map.get_mut::<T>())
    }

    pub fn insert<T>(&mut self, uid: Uid, v: T) -> Option<T>
    where
        T: 'static,
    {
        self.inner.entry(uid).or_default().insert(v)
    }

    pub fn remove<T>(&mut self, uid: Uid) -> Option<T>
    where
        T: 'static,
    {
        self.inner.get_mut(&uid).and_then(|v| v.remove::<T>())
    }

    pub fn entry<T>(&mut self, uid: Uid) -> ValEntry<'_, T> {
        ValEntry::new(uid, self.inner.entry(uid).or_insert(Map::default()))
    }
}

pub struct ValEntry<'a, T> {
    uid: Uid,

    map: &'a mut Map,

    marker: PhantomData<T>,
}

impl<'a, T> ValEntry<'a, T> {
    pub fn new(uid: Uid, map: &'a mut Map) -> Self {
        Self {
            uid,
            map,
            marker: PhantomData::default(),
        }
    }
}

impl<'a, T> ValEntry<'a, T>
where
    T: 'static,
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

/// Extension trait of [`ValueService`].
pub trait ValServiceExt {
    fn val<V: 'static>(&self, uid: Uid) -> Result<&V, Error>;

    fn val_mut<V: 'static>(&mut self, uid: Uid) -> Result<&mut V, Error>;
}

impl ValServiceExt for ValService {
    fn val<V: 'static>(&self, uid: Uid) -> Result<&V, Error> {
        self.get(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    fn val_mut<V: 'static>(&mut self, uid: Uid) -> Result<&mut V, Error> {
        self.get_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }
}
