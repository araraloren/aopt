use crate::ctx::Store;
use crate::ext::ServicesExt;
use crate::ser::Services;
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
impl<Set, Val> Store<Set, Val> for Action
where
    Val: 'static,
{
    type Ret = ();

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        _: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Val>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let has_value = val.is_some();

        trace!("Store the value of {{{uid}}} ==> {:?}", raw);
        // Set the value if return Some(Value)
        if let Some(val) = val {
            let raw_ser = ser.ser_rawval_mut()?;

            if let Some(raw) = raw {
                raw_ser.push(uid, raw.clone());
            }

            let val_ser = ser.ser_val_mut()?;

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

    /// Raw value will parsed as [`u64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Uint,

    /// Raw value will parsed as [`f64`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Flt,

    /// Raw value will parsed as [`String`] when using default [`handler`](crate::ser::InvokeService::fallback).
    Str,

    /// The value of option will set to true when using default [`handler`](crate::ser::InvokeService::fallback).
    Noa,

    /// Raw value will parsed as [`PathBuf`](std::path::PathBuf) when using default [`handler`](crate::ser::InvokeService::fallback).
    Path,

    Null,
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
            Assoc::Uint => {
                write!(f, "Assoc::Uint")
            }
            Assoc::Flt => {
                write!(f, "Assoc::Flt")
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
