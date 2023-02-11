use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::value::raw2str;
use crate::value::RawValParser;
use crate::Error;

pub trait Serialize
where
    Self: serde::Serialize,
{
    type Output;

    type Error: Into<crate::Error>;

    fn serialize_to(&self) -> Result<Self::Output, Self::Error>;
}

pub trait Deserialize<'a>
where
    Self: serde::Deserialize<'a>,
{
    type Error: Into<Error>;

    fn deserialize_from(str: &'a str) -> Result<Self, Self::Error>;
}

pub struct Serde<T>(T);

impl<T> Serde<T> {
    pub fn replace(&mut self, val: T) -> T {
        std::mem::replace(&mut self.0, val)
    }
}

impl<T: Default> Serde<T> {
    pub fn take(&mut self) -> T {
        std::mem::take(&mut self.0)
    }
}

impl<T: Debug> Debug for Serde<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Serde").field(&self.0).finish()
    }
}

impl<T: Display> Display for Serde<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Serde({})", self.0)
    }
}

impl<T: Clone> Clone for Serde<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for Serde<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: PartialEq<T>> PartialEq<Self> for Serde<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for Serde<T> {}

impl<T: PartialOrd<T>> PartialOrd<Self> for Serde<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord> Ord for Serde<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Hash> Hash for Serde<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Serde<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> RawValParser for Serde<T>
where
    T: for<'a> Deserialize<'a>,
{
    type Error = Error;

    fn parse(val: Option<&crate::RawVal>, _: &Ctx) -> Result<Self, Self::Error> {
        let string = raw2str(val)?;

        Ok(Serde(T::deserialize_from(string).map_err(|e| e.into())?))
    }
}
