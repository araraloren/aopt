use crate::astr;
use crate::map::AnyMap;
use crate::map::Entry;
use crate::map::ErasedTy;
use crate::ser::Service;
use crate::Error;

#[derive(Default)]
pub struct UsrValService(AnyMap);

impl Service for UsrValService {
    fn service_name() -> crate::Str {
        astr("UsrValService")
    }
}

impl UsrValService {
    pub fn new() -> Self {
        Self(AnyMap::default())
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
        self.0.contain::<T>()
    }

    pub fn insert<T: ErasedTy>(&mut self, value: T) -> Option<T> {
        self.0.insert(value)
    }

    pub fn remove<T: ErasedTy>(&mut self) -> Option<T> {
        self.0.remove::<T>()
    }

    pub fn get<T: ErasedTy>(&self) -> Option<&T> {
        self.0.get::<T>()
    }

    pub fn get_mut<T: ErasedTy>(&mut self) -> Option<&mut T> {
        self.0.get_mut::<T>()
    }

    pub fn val<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.0.ty::<T>()
    }

    pub fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.0.ty_mut::<T>()
    }

    pub fn entry<T: ErasedTy>(&mut self) -> Entry<'_, T> {
        self.0.entry::<T>()
    }
}
