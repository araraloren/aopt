use crate::map::Map;
use crate::{HashMap, Uid};

use super::Services;

#[derive(Debug, Default)]
pub struct ValServices {
    inner: HashMap<Uid, (Map, Map)>,
}

pub trait Adapter<T> {
    fn get(&self, ser: &Map) -> Option<&T>;

    fn get_mut(&mut self, ser: &mut Map) -> Option<&mut T>;

    fn set(&mut self, ser: &mut Map, val: T);
}

pub struct ValAdapter<T> {
    get: Box<dyn Fn(&Map) -> Option<&T>>,
    r#mut: Box<dyn FnMut(&mut Map) -> Option<&mut T>>,
    set: Box<dyn FnMut(&mut Map, T)>,
}

impl<T> Default for ValAdapter<T>
where
    T: 'static,
{
    fn default() -> Self {
        fn default_get<T: 'static>(map: &Map) -> Option<&T> {
            None
        }

        fn default_mut<T: 'static>(map: &mut Map) -> Option<&mut T> {
            None
        }

        fn default_set<T: 'static>(map: &mut Map, val: T) {}

        Self::new(default_get, default_mut, default_set)
    }
}

impl<T> ValAdapter<T>
where
    T: 'static,
{
    pub fn new<G, M, S>(get: G, r#mut: M, set: S) -> Self
    where
        G: Fn(&Map) -> Option<&T> + 'static,
        M: FnMut(&mut Map) -> Option<&mut T> + 'static,
        S: FnMut(&mut Map, T) + 'static,
    {
        Self {
            get: Box::new(get),
            r#mut: Box::new(r#mut),
            set: Box::new(set),
        }
    }

    fn get<'a>(&self, ser: &'a Map) -> Option<&'a T> {
        (self.get)(ser)
    }

    fn r#mut<'a>(&mut self, ser: &'a mut Map) -> Option<&'a mut T> {
        (self.r#mut)(ser)
    }

    fn set(&mut self, ser: &mut Map, val: T) {
        (self.set)(ser, val)
    }
}
