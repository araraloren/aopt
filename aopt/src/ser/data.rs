use crate::astr;
use crate::map::AnyMap;
use crate::map::Entry;
use crate::ser::Service;
use crate::Error;

#[derive(Default)]
pub struct DataService(AnyMap);

impl Service for DataService {
    fn service_name() -> crate::Str {
        astr("DataService")
    }
}

impl DataService {
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

    pub fn contain<T: 'static>(&self) -> bool {
        self.0.contain::<T>()
    }

    pub fn insert<T: 'static>(&mut self, value: T) -> Option<T> {
        self.0.insert(value)
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.0.remove::<T>()
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0.get::<T>()
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0.get_mut::<T>()
    }

    pub fn data<T: 'static>(&self) -> Result<&T, Error> {
        self.0.ty::<T>()
    }

    pub fn data_mut<T: 'static>(&mut self) -> Result<&mut T, Error> {
        self.0.ty_mut::<T>()
    }

    pub fn entry<T: 'static>(&mut self) -> Entry<'_, T> {
        self.0.entry::<T>()
    }
}
