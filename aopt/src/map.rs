use std::any::Any;
use std::any::TypeId;
use std::collections::hash_map::Entry as MapEntry;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::typeid;
use crate::Error;
use crate::HashMap;

#[derive(Default)]
pub struct Map(HashMap<TypeId, Box<dyn Any>>);

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Map").field(&"{...}").finish()
    }
}

impl Map {
    pub fn with<T>(mut self, value: T) -> Self
    where
        T: 'static,
    {
        self.0.insert(typeid::<T>(), Box::new(value));
        self
    }

    pub fn new() -> Self {
        Self(HashMap::default())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn contain<T>(&self) -> bool
    where
        T: 'static,
    {
        self.0.contains_key(&typeid::<T>())
    }

    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static,
    {
        self.0
            .insert(typeid::<T>(), Box::new(value))
            .and_then(|v| v.downcast().ok().map(|v| *v))
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: 'static,
    {
        self.0
            .remove(&typeid::<T>())
            .and_then(|v| v.downcast().ok().map(|v| *v))
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.0.get(&typeid::<T>()).and_then(|v| v.downcast_ref())
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.0
            .get_mut(&typeid::<T>())
            .and_then(|v| v.downcast_mut())
    }
    pub fn entry<T>(&mut self) -> Entry<'_, T>
    where
        T: 'static,
    {
        Entry::new(self.0.entry(typeid::<T>()))
    }
}

pub struct Entry<'a, T> {
    inner: MapEntry<'a, TypeId, Box<dyn Any>>,

    marker: PhantomData<T>,
}

impl<'a, T> Entry<'a, T>
where
    T: 'static,
{
    pub fn new(entry: MapEntry<'a, TypeId, Box<dyn Any>>) -> Self {
        Self {
            inner: entry,
            marker: PhantomData::default(),
        }
    }

    pub fn key(&self) -> &TypeId {
        self.inner.key()
    }

    pub fn or_insert(self, val: T) -> &'a mut T {
        self.inner
            .or_insert(Box::new(val))
            .downcast_mut::<T>()
            .unwrap()
    }

    pub fn or_insert_with<F>(self, f: F) -> &'a mut T
    where
        F: FnOnce() -> T,
    {
        self.or_insert(f())
    }

    pub fn or_insert_with_key<F>(self, f: F) -> &'a mut T
    where
        F: FnOnce(&TypeId) -> T,
    {
        let val = f(self.key());
        self.or_insert(val)
    }

    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut T),
    {
        self.inner = self.inner.and_modify(|v| f(v.downcast_mut::<T>().unwrap()));
        self
    }
}

pub trait MapExt {
    fn ty<T>(&self) -> Result<&T, Error>
    where
        T: 'static;

    fn ty_mut<T>(&mut self) -> Result<&mut T, Error>
    where
        T: 'static;
}

impl MapExt for Map {
    fn ty<T>(&self) -> Result<&T, Error>
    where
        T: 'static,
    {
        debug_assert!(self.contain::<T>(), "Unknown type for AnyMap");
        self.get::<T>().ok_or_else(|| {
            Error::raise_error(format!("Unknown type for AnyMap: {:?}", typeid::<T>()))
        })
    }

    fn ty_mut<T>(&mut self) -> Result<&mut T, Error>
    where
        T: 'static,
    {
        debug_assert!(self.contain::<T>(), "Unknown type for AnyMap");
        self.get_mut::<T>().ok_or_else(|| {
            Error::raise_error(format!("Unknown type for AnyMap: {:?}", typeid::<T>()))
        })
    }
}

#[derive(Default, Clone)]
pub struct RcMap(Rc<Map>);

impl Debug for RcMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RcMap").field(&self.0).finish()
    }
}

impl RcMap {
    pub fn with<T>(mut self, value: T) -> Self
    where
        T: 'static,
    {
        self.insert(value);
        self
    }

    pub fn new() -> Self {
        Self(Rc::new(Map::new()))
    }

    fn inner_mut(&mut self) -> Option<&mut Map> {
        let inner = Rc::get_mut(&mut self.0);
        debug_assert!(inner.is_some(), "Can not get mutable reference of RcMap !?");
        inner
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clr(&mut self) {
        if let Some(v) = self.inner_mut() {
            v.clear();
        }
    }

    pub fn contain<T>(&self) -> bool
    where
        T: 'static,
    {
        self.0.contain::<T>()
    }

    pub fn remove<T>(&mut self) -> Option<T>
    where
        T: 'static,
    {
        self.inner_mut().and_then(|v| v.remove::<T>())
    }

    pub fn insert<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static,
    {
        self.inner_mut().and_then(|v| v.insert(value))
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.0.get::<T>()
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static,
    {
        self.inner_mut().and_then(|v| v.get_mut::<T>())
    }
}

pub trait RcMapExt {
    fn ty<T>(&self) -> Result<&T, Error>
    where
        T: 'static;

    fn ty_mut<T>(&mut self) -> Result<&mut T, Error>
    where
        T: 'static;
}

impl RcMapExt for RcMap {
    fn ty<T>(&self) -> Result<&T, Error>
    where
        T: 'static,
    {
        self.get::<T>().ok_or_else(|| {
            Error::raise_error(format!("Unknown type for RefAnyMap: {:?}", typeid::<T>()))
        })
    }

    fn ty_mut<T>(&mut self) -> Result<&mut T, Error>
    where
        T: 'static,
    {
        self.get_mut::<T>().ok_or_else(|| {
            Error::raise_error(format!("Unknown type for RefAnyMap: {:?}", typeid::<T>()))
        })
    }
}

#[cfg(test)]
mod test {
    use super::Map;
    use super::MapExt;

    #[test]
    fn test_typed_value_map() {
        let mut map = Map::new();

        // set initialize value for map
        map = map.with(42i32);
        map = map.with(21u32);

        assert!(!map.is_empty());
        assert_ne!(map.len(), 0);

        // check the type in the map
        assert!(map.contain::<i32>());
        assert!(map.contain::<u32>());
        assert!(!map.contain::<i64>());

        #[derive(Debug, PartialEq)]
        struct Widget(i64);

        // insert a self-define type
        map.insert(Widget(1));

        // check type Widget
        assert!(map.contain::<Widget>());

        // check the value get from map
        assert_eq!(map.get::<Widget>(), Some(&Widget(1)));
        assert_eq!(map.ty::<Widget>().unwrap(), &Widget(1));

        // modify the value in the map
        if let Some(v) = map.get_mut::<Widget>() {
            v.0 = 2
        };
        assert_eq!(map.get::<Widget>(), Some(&Widget(2)));
        if let Ok(v) = map.ty_mut::<u32>() {
            *v = 42;
        }
        assert_eq!(map.ty::<u32>().unwrap(), &42);

        // remove the self-define type
        map.remove::<Widget>();
        assert!(!map.contain::<Widget>());

        // clear the map
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }
}
