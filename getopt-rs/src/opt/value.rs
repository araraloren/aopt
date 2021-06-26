
use std::any::Any;
use std::convert::From;

#[derive(Debug)]
pub enum Value {
    Int(i64),

    Uint(u64),

    Flt(f64),

    Str(String),

    Bool(bool),

    Array(Vec<String>),

    Any(Box<dyn Any>),

    Null,
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Self::Uint(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Flt(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::Str(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::Str(String::from(v))
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<Vec<String>> for Value {
    fn from(v: Vec<String>) -> Self {
        Self::Array(v)
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Value {
    pub fn from_any<T: Any>(t: Box<T>) -> Self {
        Self::Any(t)
    }

    pub fn as_int(&self) -> Option<&i64> {
        match self {
            Self::Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_uint(&self) -> Option<&u64> {
        match self {
            Self::Uint(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_flt(&self) -> Option<&f64> {
        match self {
            Self::Flt(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&String> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<&Vec<String>> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_any(&self) -> Option<&Box<dyn Any>> {
        match self {
            Self::Any(v) => Some(v),
            _ => None,
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        match self {
            Self::Any(v) => v.as_ref().downcast_ref::<T>(),
            _ => None,
        }
    }

    pub fn as_int_mut(&mut self) -> Option<&mut i64> {
        match self {
            Self::Int(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_uint_mut(&mut self) -> Option<&mut u64> {
        match self {
            Self::Uint(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_flt_mut(&mut self) -> Option<&mut f64> {
        match self {
            Self::Flt(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vec_mut(&mut self) -> Option<&mut Vec<String>> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_any_mut(&mut self) -> Option<&mut Box<dyn Any>> {
        match self {
            Self::Any(v) => Some(v),
            _ => None,
        }
    }

    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        match self {
            Self::Any(v) => v.as_mut().downcast_mut::<T>(),
            _ => None,
        }
    }
}