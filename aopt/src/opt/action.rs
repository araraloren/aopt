use crate::ctx::Store;
use crate::ext::ServicesExt;
use crate::ser::Services;
use crate::Error;
use crate::RawVal;
use crate::Uid;
use tracing::trace;

///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Action {
    Set,

    App,

    Pop,

    Cnt,

    Null,
}

impl Default for Action {
    fn default() -> Self {
        Action::Null
    }
}

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
            Action::Null => {
                write!(f, "Action::Null")
            }
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Assoc {
    Bool,

    Int,

    Uint,

    Flt,

    Str,

    Noa,

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
            Assoc::Null => {
                write!(f, "Assoc::Null")
            }
        }
    }
}
