use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ARef;

pub fn astr<T: Into<AStr>>(value: T) -> AStr {
    value.into()
}

/// A simple wrapper of [`ARef`]\<str\>.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AStr(ARef<str>);

impl AStr {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl Clone for AStr {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for AStr {
    fn default() -> Self {
        Self("".into())
    }
}

impl<'a> From<&'a str> for AStr {
    fn from(value: &'a str) -> Self {
        AStr(ARef::from(value))
    }
}

impl From<String> for AStr {
    fn from(value: String) -> Self {
        AStr(ARef::from(value))
    }
}

impl<'a> From<&'a AStr> for AStr {
    fn from(value: &'a AStr) -> Self {
        value.clone()
    }
}

impl From<AStr> for String {
    fn from(value: AStr) -> Self {
        String::from(value.as_str())
    }
}

impl<'a> From<&'a AStr> for String {
    fn from(value: &'a AStr) -> Self {
        String::from(value.as_str())
    }
}

impl<'a> From<Cow<'a, str>> for AStr {
    fn from(value: Cow<'a, str>) -> Self {
        Self(ARef::from(value))
    }
}

impl Deref for AStr {
    type Target = ARef<str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for AStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl DerefMut for AStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for AStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for AStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for AStr {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for AStr {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for AStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
struct StrVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for StrVisitor {
    type Value = AStr;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("AStr")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::from(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::from(String::from_utf8(v.to_vec()).map_err(
            |e| serde::de::Error::custom(format!("Invalid utf8 string for AStr: {}", e)),
        )?))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::from(v))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(StrVisitor)
    }
}

#[cfg(target_family = "windows")]
pub fn split_once<'a>(str: &'a OsStr, ch: char) -> Option<(Cow<'a, OsStr>, Cow<'a, OsStr>)> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let enc = str.encode_wide();
    let mut buf = [0; 1];
    let sep = ch.encode_utf16(&mut buf);
    let enc = enc.collect::<Vec<u16>>();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .map(|(i, _)| {
            (
                Cow::Owned(OsString::from_wide(&enc[0..i])),
                Cow::Owned(OsString::from_wide(&enc[i + 1..])),
            )
        })
}

#[cfg(any(target_family = "wasm", target_family = "unix"))]
pub fn split_once<'a>(str: &'a OsStr, ch: char) -> Option<(Cow<'a, OsStr>, Cow<'a, OsStr>)> {
    #[cfg(target_family = "unix")]
    use std::os::unix::ffi::OsStrExt;
    #[cfg(target_family = "wasm")]
    use std::os::wasi::ffi::OsStrExt;

    let enc = str.as_bytes();
    let mut buf = [0; 1];
    let sep = ch.encode_utf8(&mut buf).as_bytes();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .map(|(i, _)| {
            (
                Cow::Borrowed(OsStr::from_bytes(&enc[0..i])),
                Cow::Borrowed(OsStr::from_bytes(&enc[i + 1..])),
            )
        })
}

pub fn osstr_to_str_i<'a>(val: &[&'a OsStr], i: usize) -> Option<Cow<'a, str>> {
    val.get(i)
        .and_then(|v| v.to_str().map(|v| Cow::Borrowed(v)))
}

pub fn display_of_str(val: Option<&str>) -> String {
    if let Some(val) = val {
        format!("Some({})", val)
    } else {
        "None".to_string()
    }
}

pub fn display_of_osstr(val: Option<&OsStr>) -> String {
    if let Some(val) = val {
        format!("Some({})", std::path::Path::new(val).display())
    } else {
        "None".to_string()
    }
}

pub trait CowOsStrUtils<'a> {
    fn split_once(&self, sep: char) -> Option<(Cow<'a, OsStr>, Cow<'a, OsStr>)>;

    fn to_str(&self, func: impl Fn(&str) -> &str) -> Option<Cow<'a, str>>;
}

impl<'a> CowOsStrUtils<'a> for Cow<'a, OsStr> {
    fn split_once(&self, sep: char) -> Option<(Cow<'a, OsStr>, Cow<'a, OsStr>)> {
        match self {
            Cow::Borrowed(v) => split_once(v, sep),
            Cow::Owned(v) => split_once(&v, sep)
                .map(|(a, b)| (Cow::Owned(a.into_owned()), Cow::Owned(b.into_owned()))),
        }
    }

    fn to_str(&self, func: impl Fn(&str) -> &str) -> Option<Cow<'a, str>> {
        match &self {
            Cow::Borrowed(v) => v.to_str().map(func).map(Cow::Borrowed),
            Cow::Owned(v) => v.to_str().map(func).map(String::from).map(Cow::Owned),
        }
    }
}

pub trait CowStrUtils<'a> {
    fn split_at(&self, mid: usize) -> (Cow<'a, str>, Cow<'a, str>);

    fn to_os_str(self) -> Cow<'a, OsStr>;
}

impl<'a> CowStrUtils<'a> for Cow<'a, str> {
    fn split_at(&self, mid: usize) -> (Cow<'a, str>, Cow<'a, str>) {
        match self {
            Cow::Borrowed(v) => {
                let (a, b) = v.split_at(mid);

                (Cow::Borrowed(a), Cow::Borrowed(b))
            }
            Cow::Owned(v) => {
                let (a, b) = v.split_at(mid);

                (Cow::Owned(a.to_string()), Cow::Owned(b.to_string()))
            }
        }
    }

    fn to_os_str(self) -> Cow<'a, OsStr> {
        match self {
            Cow::Borrowed(v) => Cow::Borrowed(OsStr::new(v)),
            Cow::Owned(v) => Cow::Owned(OsString::from(v)),
        }
    }
}
