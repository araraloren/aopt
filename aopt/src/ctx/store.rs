use crate::opt::Opt;
use crate::prelude::ErasedTy;
use crate::set::SetOpt;
use crate::Error;
use crate::RawVal;
use crate::Uid;

/// The [`Store`] processer save the value of given option into
/// [`AnyValService`](crate::ser::AnyValService) and [`RawValServices`](crate::ser::RawValService).
pub trait Store<Set, Ser, Value> {
    type Ret;
    type Error: Into<Error>;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Ser,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Self::Ret, Self::Error>;
}

#[cfg(not(feature = "sync"))]
impl<Func, Set, Ser, Value, Ret, Err> Store<Set, Ser, Value> for Func
where
    Err: Into<Error>,
    Func: FnMut(Uid, &mut Set, &mut Ser, Option<&RawVal>, Option<Value>) -> Result<Ret, Err>,
{
    type Ret = Ret;
    type Error = Err;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Ser,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Self::Ret, Self::Error> {
        (self)(uid, set, ser, raw, val)
    }
}
#[cfg(feature = "sync")]
impl<Func, Set, Ser, Value, Ret, Err> Store<Set, Ser, Value> for Func
where
    Err: Into<Error>,
    Func: FnMut(Uid, &mut Set, &mut Ser, Option<&RawVal>, Option<Value>) -> Result<Ret, Err>
        + Send
        + Sync,
{
    type Ret = Ret;
    type Error = Err;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Ser,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Self::Ret, Self::Error> {
        (self)(uid, set, ser, raw, val)
    }
}

/// Null store, do nothing. See [`Action`](crate::opt::Action) for default store.
pub struct NullStore;

impl<Set, Ser, Value> Store<Set, Ser, Value> for NullStore {
    type Ret = bool;

    type Error = Error;

    fn process(
        &mut self,
        _: Uid,
        _: &mut Set,
        _: &mut Ser,
        _: Option<&RawVal>,
        _: Option<Value>,
    ) -> Result<Self::Ret, Self::Error> {
        Ok(true)
    }
}

/// Vector store, append the value to the [`AnyValService`](crate::ser::AnyValService)
/// if option's action is Action::App.
/// See [`Action`](crate::opt::Action) for default store.
pub struct VecStore;

impl<Set, Ser, Value: ErasedTy> Store<Set, Ser, Vec<Value>> for VecStore
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
{
    type Ret = bool;

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        _: &mut Ser,
        raw: Option<&RawVal>,
        val: Option<Vec<Value>>,
    ) -> Result<Self::Ret, Self::Error> {
        let has_value = val.is_some();

        // Set the value if return Some(Value)
        if let Some(val) = val {
            if let Some(opt) = set.get_mut(uid) {
                let act = *opt.action();
                let (raw_handler, handler) = opt.accessor_mut().handlers();

                if act.is_app() {
                    if let Some(raw) = raw {
                        raw_handler.push(raw.clone());
                    }
                    for value in val {
                        handler.push(value);
                    }
                } else {
                    panic!("Option action is not App, but set a vector vector")
                }
            }
        }

        Ok(has_value)
    }
}
