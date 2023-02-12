use crate::ctx::Store;
use crate::map::ErasedTy;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::value::AnyValue;
use crate::Error;
use crate::RawVal;
use crate::Uid;

use super::Opt;

/// The default action type for option value saving, see [`Action::process`].
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    /// Save the value in [`handler`](AnyValue).
    pub fn store1<U: ErasedTy>(&self, val: Option<U>, handler: &mut AnyValue) -> bool {
        crate::trace_log!(
            "Saving value {:?}({:?}) [ty = {:?}] in store1",
            val,
            self,
            crate::typeid::<Vec<U>>()
        );
        if let Some(val) = val {
            match self {
                Action::Set => {
                    handler.set(vec![val]);
                }
                Action::App => {
                    handler.push(val);
                }
                Action::Pop => {
                    handler.pop::<U>();
                }
                Action::Cnt => {
                    handler.entry::<u64>().or_insert(vec![0])[0] += 1;
                }
                Action::Clr => {
                    handler.remove::<U>();
                }
                Action::Null => {
                    // NOTHING
                }
            }
            crate::trace_log!("After saving handler: {:?}", handler);
            true
        } else {
            false
        }
    }

    /// Save the value in [`handler`](AnyValue) and raw value in `raw_handler`.
    pub fn store2<U: ErasedTy>(
        &self,
        raw: Option<&RawVal>,
        val: Option<U>,
        raw_handler: &mut Vec<RawVal>,
        handler: &mut AnyValue,
    ) -> bool {
        let ret = self.store1(val, handler);

        if ret {
            if let Some(raw) = raw {
                raw_handler.push(raw.clone());
            }
        }
        ret
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Null
    }
}

/// Default store using for store value to [`ValStorer`](crate::value::ValStorer).
/// It will store `RawVal` and `Val` if `val` is `Some(Val)`, otherwise do nothing.
///
/// Note: The [`ValStorer`](crate::value::ValStorer) internal using an [`vec`] saving the option value.
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
    SetOpt<Set>: Opt,
    Set: crate::set::Set,
{
    type Ret = bool;

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        _: &mut Ser,
        raw: Option<&RawVal>,
        val: Option<Val>,
    ) -> Result<Self::Ret, Self::Error> {
        let opt = set.opt_mut(uid)?;

        crate::trace_log!("Store the value of {} ==> {:?}", opt.name().clone(), raw);

        let (raw_handler, handler) = opt.accessor_mut().handlers();

        // Set the value if return Some(Value)
        Ok(self.store2(raw, val, raw_handler, handler))
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
