use std::any::type_name;
use std::any::Any;
use std::any::TypeId;
use std::collections::hash_map::Entry as MapEntry;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::typeid;
use crate::Error;
use crate::HashMap;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub trait ErasedTy: Any + Sync + Send + 'static { }

        impl<T:  Any + Sync + Send + 'static> ErasedTy for T { }

        type BoxedAny = Box<dyn Any + Send + Sync>;

        #[derive(Default)]
        pub struct AnyMap(HashMap<TypeId, BoxedAny>);
    }
    else {
        pub trait ErasedTy: Any + 'static { }

        impl<T: Any + 'static> ErasedTy for T { }

        type BoxedAny = Box<dyn Any>;

        #[derive(Default)]
        pub struct AnyMap(HashMap<TypeId, BoxedAny>);
    }
}

impl Debug for AnyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AnyMap").field(&"{...}").finish()
    }
}

impl AnyMap {
    pub fn with<T: ErasedTy>(mut self, value: T) -> Self {
        self.0.insert(typeid::<T>(), Box::new(value));
        self
    }
}

impl AnyMap {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contain<T: ErasedTy>(&self) -> bool {
        self.0.contains_key(&typeid::<T>())
    }

    pub fn insert<T: ErasedTy>(&mut self, value: T) -> Option<T> {
        self.0
            .insert(typeid::<T>(), Box::new(value))
            .and_then(|v| v.downcast().ok().map(|v| *v))
    }

    pub fn remove<T: ErasedTy>(&mut self) -> Option<T> {
        self.0
            .remove(&typeid::<T>())
            .and_then(|v| v.downcast().ok().map(|v| *v))
    }

    pub fn get<T: ErasedTy>(&self) -> Option<&T> {
        self.0.get(&typeid::<T>()).and_then(|v| v.downcast_ref())
    }

    pub fn get_mut<T: ErasedTy>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&typeid::<T>())
            .and_then(|v| v.downcast_mut())
    }

    pub fn ty<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.get::<T>().ok_or_else(|| {
            Error::raise_error(format!(
                "Can not find type {{{:?}}} in AnyMap",
                type_name::<T>()
            ))
        })
    }

    pub fn ty_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.get_mut::<T>().ok_or_else(|| {
            Error::raise_error(format!(
                "Can not find type {{{:?}}} in AnyMap",
                type_name::<T>()
            ))
        })
    }

    pub fn entry<T: ErasedTy>(&mut self) -> Entry<'_, T> {
        Entry::new(self.0.entry(typeid::<T>()))
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub struct Entry<'a, T> {
            inner: MapEntry<'a, TypeId, BoxedAny>,

            marker: PhantomData<T>,
        }
    }
    else {
        pub struct Entry<'a, T> {
            inner: MapEntry<'a, TypeId, BoxedAny>,

            marker: PhantomData<T>,
        }
    }
}

impl<'a, T> Entry<'a, T>
where
    T: ErasedTy,
{
    pub fn new(entry: MapEntry<'a, TypeId, BoxedAny>) -> Self {
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
            .or_insert_with(|| Box::new(val))
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
