use std::fmt::Debug;

use crate::map::ErasedTy;
use crate::Error;
use crate::HashMap;
use crate::Uid;

/// Keep the raw value in [`HashMap`] with key [`Uid`].
///
/// The service internal using [`Vec`].
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
///  let mut rvs = RawValService::<i32>::new();
///
///  rvs.push(0, 42);
///  rvs.push(0, 36);
///
///  assert_eq!(rvs.val(0)?, &36);
///  assert_eq!(rvs.vals(0)?, &vec![42, 36]);
///
///  rvs.set(0, vec![12, 24]);
///
///  assert_eq!(rvs.vals(0)?, &vec![12, 24]);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct RawValService<T: ErasedTy> {
    rets: HashMap<Uid, Vec<T>>,
}

impl<T: ErasedTy + Debug> Debug for RawValService<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawValService")
            .field("rets", &self.rets)
            .finish()
    }
}

impl<T: ErasedTy> RawValService<T> {
    pub fn new() -> Self {
        Self {
            rets: HashMap::default(),
        }
    }

    pub fn contain(&self, uid: Uid) -> bool {
        self.rets.contains_key(&uid)
    }

    pub fn get(&self, uid: Uid) -> Option<&T> {
        self.rets.get(&uid).and_then(|v| v.last())
    }

    pub fn gets(&self, uid: Uid) -> Option<&Vec<T>> {
        self.rets.get(&uid)
    }

    pub fn get_mut(&mut self, uid: Uid) -> Option<&mut T> {
        self.rets.get_mut(&uid).and_then(|v| v.last_mut())
    }

    pub fn gets_mut(&mut self, uid: Uid) -> Option<&mut Vec<T>> {
        self.rets.get_mut(&uid)
    }

    pub fn push(&mut self, uid: Uid, ret: T) -> &mut Self {
        self.rets.entry(uid).or_default().push(ret);
        self
    }

    pub fn pop(&mut self, uid: Uid) -> Option<T> {
        self.rets.entry(uid).or_default().pop()
    }

    pub fn remove(&mut self, uid: Uid) -> Option<Vec<T>> {
        self.rets.remove(&uid)
    }

    pub fn set(&mut self, uid: Uid, vals: Vec<T>) -> Option<Vec<T>> {
        self.rets.insert(uid, vals)
    }

    pub fn clear(&mut self) {
        self.rets.clear();
    }

    pub fn entry(&mut self, uid: Uid) -> std::collections::hash_map::Entry<'_, Uid, Vec<T>> {
        self.rets.entry(uid)
    }

    pub fn val(&self, uid: Uid) -> Result<&T, Error> {
        self.get(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for RawValService")))
    }

    pub fn vals(&self, uid: Uid) -> Result<&Vec<T>, Error> {
        self.gets(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for RawValService")))
    }

    pub fn val_mut(&mut self, uid: Uid) -> Result<&mut T, Error> {
        self.get_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for RawValService")))
    }

    pub fn vals_mut(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error> {
        self.gets_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {uid} for RawValService")))
    }
}
