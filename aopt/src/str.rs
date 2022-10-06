use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;

pub fn astr<T: Into<Str>>(value: T) -> Str {
    value.into()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Str(Rc<str>);

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
        Str(Rc::from(value))
    }
}

impl From<String> for Str {
    fn from(value: String) -> Self {
        Str(Rc::from(value))
    }
}

impl<'a> From<&'a Str> for Str {
    fn from(value: &'a Str) -> Self {
        value.clone()
    }
}

impl Deref for Str {
    type Target = Rc<str>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

pub trait StrJoin {
    fn join(&self, sep: &str) -> String;
}

impl StrJoin for Vec<Str> {
    fn join(&self, sep: &str) -> String {
        let mut ret = String::new();

        for (idx, item) in self.iter().enumerate() {
            if idx == self.len() - 1 {
                ret += item.as_ref();
            } else {
                ret += item.as_ref();
                ret += sep;
            }
        }
        ret
    }
}
