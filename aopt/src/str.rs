use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ARef;

pub fn astr<T: Into<AStr>>(value: T) -> AStr {
    value.into()
}

pub trait StrJoin {
    fn join(&self, sep: &str) -> String;
}

/// A simple wrapper of [`ARef`](crate::ARef)\<str\>.
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

impl StrJoin for Vec<AStr> {
    fn join(&self, sep: &str) -> String {
        match self.len() {
            0 => String::new(),
            _ => {
                let mut iter = self.iter();
                let mut ret = String::from(iter.next().unwrap().as_str());

                iter.for_each(|v| {
                    ret.push_str(sep);
                    ret.push_str(v.as_str());
                });
                ret
            }
        }
    }
}
