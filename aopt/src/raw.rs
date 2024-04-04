use crate::ARef;
use crate::AStr;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::ops::{Deref, DerefMut};

/// Raw value used when parsing command line argument, it is wrapper of [`ARef<OsStr>`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawVal(ARef<OsStr>);

impl Deref for RawVal {
    type Target = ARef<OsStr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawVal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RawVal {
    pub fn get_str(&self) -> Option<&str> {
        self.0.to_str()
    }

    pub fn into_os_string(self) -> OsString {
        self.0.to_os_string()
    }
}

impl From<OsString> for RawVal {
    fn from(v: OsString) -> Self {
        Self(v.into())
    }
}

impl<'a> From<&'a OsString> for RawVal {
    fn from(v: &'a OsString) -> Self {
        Self(v.as_os_str().into())
    }
}

impl<'a> From<&'a OsStr> for RawVal {
    fn from(v: &'a OsStr) -> Self {
        Self(v.into())
    }
}

impl From<String> for RawVal {
    fn from(v: String) -> Self {
        Self(AsRef::<OsStr>::as_ref(&v).into())
    }
}

impl<'a> From<&'a String> for RawVal {
    fn from(v: &'a String) -> Self {
        Self(AsRef::<OsStr>::as_ref(v).into())
    }
}

impl<'a> From<&'a str> for RawVal {
    fn from(v: &'a str) -> Self {
        Self(AsRef::<OsStr>::as_ref(v).into())
    }
}

impl From<AStr> for RawVal {
    fn from(v: AStr) -> Self {
        Self(AsRef::<OsStr>::as_ref(v.as_str()).into())
    }
}

impl std::fmt::Display for RawVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl AsRef<OsStr> for RawVal {
    fn as_ref(&self) -> &OsStr {
        self.0.deref()
    }
}
