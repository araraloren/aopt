pub use __raw_utf8::AStr;
pub use __raw_utf8::AString;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

#[derive(Debug, Clone, Copy)]
pub struct Just<'a, T: ?Sized>(&'a T);

pub fn just<T>(val: &T) -> Just<T> {
    Just(val)
}

impl<'a> std::fmt::Display for Just<'a, OsString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<'a> std::fmt::Display for Just<'a, OsStr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<'a> std::fmt::Display for Just<'a, Cow<'_, OsStr>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<'a> std::fmt::Display for Just<'a, Option<Cow<'_, OsStr>>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_ref() {
            Some(val) => write!(f, "Some({:?})", val),
            None => write!(f, "None"),
        }
    }
}

impl<'a> std::fmt::Display for Just<'a, String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> std::fmt::Display for Just<'a, str> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> std::fmt::Display for Just<'a, Cow<'a, str>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> std::fmt::Display for Just<'a, Option<Cow<'a, str>>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_ref() {
            Some(val) => write!(f, "Some({})", val),
            None => write!(f, "None"),
        }
    }
}

#[cfg(feature = "utf8")]
mod __raw_utf8 {
    pub type AString = String;

    pub type AStr = str;
}

#[cfg(not(feature = "utf8"))]
mod __raw_utf8 {
    use std::ffi::OsStr;
    use std::ffi::OsString;

    pub type AString = OsString;

    pub type AStr = OsStr;
}
