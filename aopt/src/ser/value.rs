use std::fmt::Debug;

use super::Service;
use crate::astr;
use crate::Error;
use crate::HashMap;
use crate::Uid;

/// Save the value with the key [`Uid`].
///
/// # Examples
/// ```rust
/// # extern crate aopt as test_crate;
/// #
/// # use test_crate::ser::ValueService;
/// # use test_crate::ser::ValueServiceExt;
/// #
/// # fn main() {
///     let mut vs = ValueService::<i32>::new();
///
///     vs.ins(0, 42);
///     vs.ins(0, 48);
///
///     assert!(vs.has(0));
///     assert_eq!(vs.val(0).unwrap(), &48);
///     assert_eq!(vs.vals(0).unwrap(), &vec![42, 48]);
/// # }
///
/// ```
#[derive(Default)]
pub struct ValueService<V> {
    rets: HashMap<Uid, Vec<V>>,
}

impl<V> Debug for ValueService<V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueService")
            .field("rets", &self.rets)
            .finish()
    }
}

impl<V> ValueService<V> {
    pub fn new() -> Self {
        Self {
            rets: HashMap::default(),
        }
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.rets.contains_key(&uid)
    }

    pub fn get(&self, uid: Uid) -> Option<&V> {
        self.rets.get(&uid).and_then(|v| v.last())
    }

    pub fn gets(&self, uid: Uid) -> Option<&Vec<V>> {
        self.rets.get(&uid)
    }

    pub fn get_mut(&mut self, uid: Uid) -> Option<&mut V> {
        self.rets.get_mut(&uid).and_then(|v| v.last_mut())
    }

    pub fn gets_mut(&mut self, uid: Uid) -> Option<&mut Vec<V>> {
        self.rets.get_mut(&uid)
    }

    pub fn ins(&mut self, uid: Uid, ret: V) -> &mut Self {
        self.rets.entry(uid).or_insert(vec![]).push(ret);
        self
    }
}

impl<V> Service for ValueService<V> {
    fn service_name() -> crate::Str {
        astr("ValueService")
    }
}

/// Extension trait of [`ValueService`].
pub trait ValueServiceExt<V> {
    fn val(&self, uid: Uid) -> Result<&V, Error>;

    fn vals(&self, uid: Uid) -> Result<&Vec<V>, Error>;

    fn val_mut(&mut self, uid: Uid) -> Result<&mut V, Error>;

    fn vals_mut(&mut self, uid: Uid) -> Result<&mut Vec<V>, Error>;
}

impl<V> ValueServiceExt<V> for ValueService<V> {
    fn val(&self, uid: Uid) -> Result<&V, Error> {
        debug_assert!(self.has(uid), "Invalid uid for ValueService");
        self.get(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    fn vals(&self, uid: Uid) -> Result<&Vec<V>, Error> {
        debug_assert!(self.has(uid), "Invalid uid for ValueService");
        self.gets(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    fn val_mut(&mut self, uid: Uid) -> Result<&mut V, Error> {
        debug_assert!(self.has(uid), "Invalid uid for ValueService");
        self.get_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }

    fn vals_mut(&mut self, uid: Uid) -> Result<&mut Vec<V>, Error> {
        debug_assert!(self.has(uid), "Invalid uid for ValueService");
        self.gets_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for ValueService")))
    }
}
