use std::any::Any;
use std::convert::From;
use std::fmt::Debug;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub trait AnyTrait : Any + Debug + Send + Sync {
            fn as_any(&self) -> &dyn Any;

            fn as_any_mut(&mut self) -> &mut dyn Any;
        }

        impl<T: Any + Debug + Send + Sync> AnyTrait for T {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    }
    else {
        pub trait AnyTrait: Any + Debug {
            fn as_any(&self) -> &dyn Any;

            fn as_any_mut(&mut self) -> &mut dyn Any;
        }

        impl<T: Any + Debug> AnyTrait for T {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }
    }
}

/// The value type of option.
#[derive(Debug)]
pub enum Value {
    Int(i64),

    Uint(u64),

    Flt(f64),

    Str(String),

    Bool(bool),

    Array(Vec<String>),

    Any(Box<dyn AnyTrait>),

    Null,
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Int(v as i64)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Self::Int(v as i64)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Self::Int(v as i64)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Self::Uint(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Self::Uint(v as u64)
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Self::Uint(v as u64)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Self::Uint(v as u64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Flt(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::Flt(v as f64)
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

impl<'a> From<&'a [String]> for Value {
    fn from(v: &'a [String]) -> Self {
        Self::Array(v.to_vec())
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Value {
    pub fn from_any<T: AnyTrait>(t: Box<T>) -> Self {
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

    pub fn as_slice(&self) -> Option<&[String]> {
        match self {
            Self::Array(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    pub fn as_any(&self) -> Option<&Box<dyn AnyTrait>> {
        match self {
            Self::Any(v) => Some(v),
            _ => None,
        }
    }

    pub fn downcast_ref<T: AnyTrait>(&self) -> Option<&T> {
        match self {
            Self::Any(v) => v.as_ref().as_any().downcast_ref::<T>(),
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

    pub fn as_slice_mut(&mut self) -> Option<&mut [String]> {
        match self {
            Self::Array(v) => Some(v.as_mut()),
            _ => None,
        }
    }

    pub fn as_any_mut(&mut self) -> Option<&mut Box<dyn AnyTrait>> {
        match self {
            Self::Any(v) => Some(v),
            _ => None,
        }
    }

    pub fn downcast_mut<T: AnyTrait>(&mut self) -> Option<&mut T> {
        match self {
            Self::Any(v) => v.as_mut().as_any_mut().downcast_mut::<T>(),
            _ => None,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Self::Int(_) => true,
            _ => false,
        }
    }

    pub fn is_uint(&self) -> bool {
        match self {
            Self::Uint(_) => true,
            _ => false,
        }
    }

    pub fn is_flt(&self) -> bool {
        match self {
            Self::Flt(_) => true,
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        match self {
            Self::Str(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_vec(&self) -> bool {
        match self {
            Self::Array(_) => true,
            _ => false,
        }
    }

    pub fn is_any(&self) -> bool {
        match self {
            Self::Any(_) => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Null;
    }

    /// Add string if current value is [`Value::Array`].
    pub fn add_str(&mut self, string: String) -> &mut Self {
        match self {
            Self::Array(v) => {
                v.push(string);
            }
            _ => {}
        }
        self
    }

    /// Append string if current value is [`Value::Array`].
    pub fn app_str(&mut self, values: &[String]) -> &mut Self {
        match self {
            Self::Array(v) => {
                for value in values {
                    v.push(value.clone());
                }
            }
            _ => {}
        }
        self
    }

    /// Return a new value contains inner strings and other's strings if type is [`Value::Array`].
    ///
    /// It will take the ownership of strings of current value.
    pub fn merge(&mut self, other: &Self) -> Option<Self> {
        match self {
            Self::Array(values) => {
                let mut ret = vec![];

                if let Some(ov) = other.as_vec() {
                    for value in values {
                        ret.push(std::mem::take(value))
                    }
                    for item in ov {
                        ret.push(item.clone());
                    }
                    Some(Self::from(ret))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Return a new value contains inner strings and other's strings if type is [`Value::Array`].
    ///
    /// It will take the ownership of strings of current value.
    /// And also will take ownership of strings in other value.
    pub fn merge_mut(&mut self, other: &mut Self) -> Option<Self> {
        match self {
            Self::Array(values) => {
                let mut ret = vec![];

                if let Some(ov) = other.as_vec_mut() {
                    for value in values {
                        ret.push(std::mem::take(value))
                    }
                    for value in ov {
                        ret.push(std::mem::take(value));
                    }
                    Some(Self::from(ret))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn take_int(&mut self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(std::mem::take(v)),
            _ => None,
        }
    }

    pub fn take_uint(&mut self) -> Option<u64> {
        match self {
            Self::Uint(v) => Some(std::mem::take(v)),
            _ => None,
        }
    }

    pub fn take_flt(&mut self) -> Option<f64> {
        match self {
            Self::Flt(v) => Some(std::mem::take(v)),
            _ => None,
        }
    }

    pub fn take_str(&mut self) -> Option<String> {
        match self {
            Self::Str(v) => Some(std::mem::take(v)),
            _ => None,
        }
    }

    pub fn take_bool(&mut self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(std::mem::take(v)),
            _ => None,
        }
    }

    pub fn take_vec(&mut self) -> Option<Vec<String>> {
        match self {
            Self::Array(v) => Some(std::mem::take(v)),
            _ => None,
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::Null | Self::Any(_) => Self::Null,
            Self::Int(v) => Self::Int(*v),
            Self::Uint(v) => Self::Uint(*v),
            Self::Flt(v) => Self::Flt(*v),
            Self::Str(v) => Self::Str(v.clone()),
            Self::Bool(v) => Self::Bool(*v),
            Self::Array(v) => Self::Array(v.clone()),
        }
    }
}

pub struct CloneHelper(Box<dyn Fn(&dyn Any) -> Box<dyn Any>>);

impl Debug for CloneHelper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyCloneHelper")
            .field("Fn", &String::from("..."))
            .finish()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Null => match other {
                Self::Null => true,
                _ => false,
            },
            Value::Int(v) => match other {
                Self::Int(ov) => *v == *ov,
                _ => false,
            },
            Value::Uint(v) => match other {
                Self::Uint(ov) => *v == *ov,
                _ => false,
            },
            Value::Flt(v) => match other {
                Self::Flt(ov) => *v == *ov,
                _ => false,
            },
            Value::Str(v) => match other {
                Self::Str(ov) => v == ov,
                _ => false,
            },
            Value::Bool(v) => match other {
                Self::Bool(ov) => *v == *ov,
                _ => false,
            },
            Value::Array(v) => match other {
                Self::Array(ov) => v == ov,
                _ => false,
            },
            Value::Any(_) => false,
        }
    }
}
