use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::Arc;

pub fn astr<T: Into<Str>>(value: T) -> Str {
    value.into()
}

pub trait StrJoin {
    fn join(&self, sep: &str) -> String;
}

/// A simple wrapper of [`Arc`](crate::Arc)\<str\>.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str(Arc<str>);

impl Str {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl Clone for Str {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for Str {
    fn default() -> Self {
        Self("".into())
    }
}

impl<'a> From<&'a str> for Str {
    fn from(value: &'a str) -> Self {
        Str(Arc::from(value))
    }
}

impl From<String> for Str {
    fn from(value: String) -> Self {
        Str(Arc::from(value))
    }
}

impl<'a> From<&'a Str> for Str {
    fn from(value: &'a Str) -> Self {
        value.clone()
    }
}

impl Deref for Str {
    type Target = Arc<str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl DerefMut for Str {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for Str {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Str {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Serialize for Str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

struct StrVisitor;

impl<'de> Visitor<'de> for StrVisitor {
    type Value = Str;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Str")
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
            |e| serde::de::Error::custom(format!("Invalid utf8 string for Str: {}", e)),
        )?))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::from(v))
    }
}

impl<'de> Deserialize<'de> for Str {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(StrVisitor)
    }
}

impl StrJoin for Vec<Str> {
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
