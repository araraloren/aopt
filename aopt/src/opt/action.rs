use crate::ctx::Store;
use crate::map::ErasedTy;
use crate::ser::ServicesExt;
use crate::Error;
use crate::RawVal;
use crate::Uid;
use tracing::trace;

/// The default action type for option value saving, see [`Action::process`].
#[non_exhaustive]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Action {
    Set,

    App,

    Pop,

    Cnt,

    Clr,

    Null,
}

impl Action {
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set)
    }

    pub fn is_app(&self) -> bool {
        matches!(self, Self::App)
    }

    pub fn is_pop(&self) -> bool {
        matches!(self, Self::Pop)
    }

    pub fn is_cnt(&self) -> bool {
        matches!(self, Self::Cnt)
    }

    pub fn is_clr(&self) -> bool {
        matches!(self, Self::Clr)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Null
    }
}

/// Default store using for store value to [`Service`](crate::ser::Service).
/// It will store `RawVal` and `Val` if `val` is `Some(Val)`, otherwise do nothing.
///
/// Note: The [`ValService`](crate::ser::ValService) internal using an [`vec`] saving the option value.
///
/// * [`Action::Set`] : Set the option value to `vec![ val ]`.
///
/// * [`Action::App`] : Append the value to value vector.
///
/// * [`Action::Pop`] : Pop last value from value vector.
///
/// * [`Action::Cnt`] : Count the value and save the count as `vec![cnt]`.
///
/// * [`Action::Clr`] : Clear all the value from value vector.
///
/// * [`Action::Null`] : Do nothing.
impl<Set, Ser, Val> Store<Set, Ser, Val> for Action
where
    Val: ErasedTy,
    Ser: ServicesExt,
{
    type Ret = ();

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        _: &mut Set,
        ser: &mut Ser,
        raw: Option<&RawVal>,
        val: Option<Val>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let has_value = val.is_some();

        trace!("Store the value of {{{uid}}} ==> {:?}", raw);
        // Set the value if return Some(Value)
        if let Some(val) = val {
            let raw_ser = ser.ser_rawval_mut();

            if let Some(raw) = raw {
                raw_ser.push(uid, raw.clone());
            }

            let val_ser = ser.ser_val_mut();

            match self {
                Action::Set => {
                    val_ser.set(uid, vec![val]);
                }
                Action::App => {
                    val_ser.push(uid, val);
                }
                Action::Pop => {
                    val_ser.pop::<Val>(uid);
                }
                Action::Cnt => {
                    val_ser.entry::<u64>(uid).or_insert(vec![0])[0] += 1;
                }
                Action::Clr => {
                    val_ser.remove::<Val>(uid);
                }
                Action::Null => {
                    //DO NOTHING
                }
            }
        }

        Ok(has_value.then_some(()))
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Set => {
                write!(f, "Action::Set")
            }
            Action::App => {
                write!(f, "Action::App")
            }
            Action::Pop => {
                write!(f, "Action::Pop")
            }
            Action::Cnt => {
                write!(f, "Action::Cnt")
            }
            Action::Clr => {
                write!(f, "Action::Clr")
            }
            Action::Null => {
                write!(f, "Action::Null")
            }
        }
    }
}

/// The default associated value type will be parsed and save into [`Service`](crate::ser::Service),
/// see [`fallback`](crate::ser::InvokeService::fallback).
#[non_exhaustive]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Assoc {
    /// Raw value will parsed as [`bool`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Bool,

    /// Raw value will parsed as [`i64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Int,

    /// Raw value will parsed as [`i128`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Int128,

    /// Raw value will parsed as [`isize`] when using default [`handler`](crate::ser::InvokeService::fallback).
    ISize,

    /// Raw value will parsed as [`i64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Int64,

    /// Raw value will parsed as [`i32`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Int32,

    /// Raw value will parsed as [`i16`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Int16,

    /// Raw value will parsed as [`i8`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Int8,

    /// Raw value will parsed as [`u64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Uint,

    /// Raw value will parsed as [`u128`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Uint128,

    /// Raw value will parsed as [`usize`] when using default [`handler`](crate::ser::InvokeService::fallback).
    USize,

    /// Raw value will parsed as [`u64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Uint64,

    /// Raw value will parsed as [`u32`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Uint32,

    /// Raw value will parsed as [`u16`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Uint16,

    /// Raw value will parsed as [`u8`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Uint8,

    /// Raw value will parsed as [`f64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Flt,

    /// Raw value will parsed as [`f64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Flt64,

    /// Raw value will parsed as [`f32`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Flt32,

    /// Raw value will parsed as [`String`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Str,

    /// The value of option will set to true when using default [`handler`](crate::ser::InvokeService::fallback).
    Noa,

    /// Raw value will parsed as [`PathBuf`](std::path::PathBuf) when using default [`handler`](crate::ser::InvokeService::fallback).
    Path,

    Null,
}

impl Assoc {
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int)
    }

    pub fn is_int128(&self) -> bool {
        matches!(self, Self::Int128)
    }

    pub fn is_isize(&self) -> bool {
        matches!(self, Self::ISize)
    }

    pub fn is_int64(&self) -> bool {
        matches!(self, Self::Int64)
    }

    pub fn is_int32(&self) -> bool {
        matches!(self, Self::Int32)
    }

    pub fn is_int16(&self) -> bool {
        matches!(self, Self::Int16)
    }

    pub fn is_int8(&self) -> bool {
        matches!(self, Self::Int8)
    }

    pub fn is_uint(&self) -> bool {
        matches!(self, Self::Uint)
    }

    pub fn is_uint128(&self) -> bool {
        matches!(self, Self::Uint128)
    }

    pub fn is_usize(&self) -> bool {
        matches!(self, Self::USize)
    }

    pub fn is_uint64(&self) -> bool {
        matches!(self, Self::Uint64)
    }

    pub fn is_uint32(&self) -> bool {
        matches!(self, Self::Uint32)
    }

    pub fn is_uint16(&self) -> bool {
        matches!(self, Self::Uint16)
    }

    pub fn is_uint8(&self) -> bool {
        matches!(self, Self::Uint8)
    }

    pub fn is_flt(&self) -> bool {
        matches!(self, Self::Flt)
    }

    pub fn is_flt64(&self) -> bool {
        matches!(self, Self::Flt64)
    }

    pub fn is_flt32(&self) -> bool {
        matches!(self, Self::Flt32)
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str)
    }

    pub fn is_path(&self) -> bool {
        matches!(self, Self::Path)
    }

    pub fn is_noa(&self) -> bool {
        matches!(self, Self::Noa)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl Default for Assoc {
    fn default() -> Self {
        Assoc::Null
    }
}

impl std::fmt::Display for Assoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Assoc::Bool => {
                write!(f, "Assoc::Bool")
            }
            Assoc::Int => {
                write!(f, "Assoc::Int")
            }
            Assoc::Int128 => {
                write!(f, "Assoc::Int128")
            }
            Assoc::ISize => {
                write!(f, "Assoc::ISize")
            }
            Assoc::Int64 => {
                write!(f, "Assoc::Int64")
            }
            Assoc::Int32 => {
                write!(f, "Assoc::Int32")
            }
            Assoc::Int16 => {
                write!(f, "Assoc::Int16")
            }
            Assoc::Int8 => {
                write!(f, "Assoc::Int8")
            }
            Assoc::Uint => {
                write!(f, "Assoc::Uint")
            }
            Assoc::Uint128 => {
                write!(f, "Assoc::Uint128")
            }
            Assoc::USize => {
                write!(f, "Assoc::USize")
            }
            Assoc::Uint64 => {
                write!(f, "Assoc::Uint64")
            }
            Assoc::Uint32 => {
                write!(f, "Assoc::Uint32")
            }
            Assoc::Uint16 => {
                write!(f, "Assoc::Uint16")
            }
            Assoc::Uint8 => {
                write!(f, "Assoc::Uint8")
            }
            Assoc::Flt => {
                write!(f, "Assoc::Flt")
            }
            Assoc::Flt64 => {
                write!(f, "Assoc::Flt64")
            }
            Assoc::Flt32 => {
                write!(f, "Assoc::Flt32")
            }
            Assoc::Str => {
                write!(f, "Assoc::Str")
            }
            Assoc::Noa => {
                write!(f, "Assoc::Noa")
            }
            Assoc::Path => {
                write!(f, "Assoc::Path")
            }
            Assoc::Null => {
                write!(f, "Assoc::Null")
            }
        }
    }
}
