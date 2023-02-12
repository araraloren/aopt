pub use __raw_utf8::RawVal;

#[cfg(feature = "utf8")]
mod __raw_utf8 {
    use crate::Str;
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::ops::{Deref, DerefMut};

    /// Raw value used when parsing command line argument,
    /// it is wrapper of [`String`] if feature `utf8` enabled.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RawVal(String);

    impl Deref for RawVal {
        type Target = String;

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
            Some(self.0.as_str())
        }
    }

    impl TryFrom<OsString> for RawVal {
        type Error = crate::Error;

        fn try_from(value: OsString) -> Result<Self, Self::Error> {
            Ok(Self(
                value
                    .to_str()
                    .ok_or_else(|| {
                        crate::Error::raise_error(format!("Invalid utf8 for RawVal: {:?}", &value))
                    })?
                    .to_owned(),
            ))
        }
    }

    impl From<String> for RawVal {
        fn from(v: String) -> Self {
            Self(v)
        }
    }

    impl<'a> From<&'a String> for RawVal {
        fn from(v: &'a String) -> Self {
            Self(v.clone())
        }
    }

    impl<'a> From<&'a str> for RawVal {
        fn from(v: &'a str) -> Self {
            Self(v.to_owned())
        }
    }

    impl From<Str> for RawVal {
        fn from(v: Str) -> Self {
            Self(String::from(v.as_str()))
        }
    }

    impl std::fmt::Display for RawVal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl AsRef<str> for RawVal {
        fn as_ref(&self) -> &str {
            self.0.as_str()
        }
    }

    impl AsRef<OsStr> for RawVal {
        fn as_ref(&self) -> &OsStr {
            self.0.as_ref()
        }
    }
}

#[cfg(not(feature = "utf8"))]
mod __raw_utf8 {
    use crate::Str;
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::ops::{Deref, DerefMut};

    /// Raw value used when parsing command line argument,
    /// it is wrapper of [`OsString`] in default.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RawVal(OsString);

    impl Deref for RawVal {
        type Target = OsString;

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
    }

    impl From<OsString> for RawVal {
        fn from(v: OsString) -> Self {
            Self(v)
        }
    }

    impl From<String> for RawVal {
        fn from(v: String) -> Self {
            Self(OsString::from(v))
        }
    }

    impl<'a> From<&'a String> for RawVal {
        fn from(v: &'a String) -> Self {
            Self(OsString::from(v))
        }
    }

    impl<'a> From<&'a str> for RawVal {
        fn from(v: &'a str) -> Self {
            Self(OsString::from(v))
        }
    }

    impl From<Str> for RawVal {
        fn from(v: Str) -> Self {
            Self(OsString::from(v.as_str()))
        }
    }

    impl std::fmt::Display for RawVal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl AsRef<OsStr> for RawVal {
        fn as_ref(&self) -> &OsStr {
            self.as_os_str()
        }
    }
}
