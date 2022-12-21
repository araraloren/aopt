use crate::ext::ServicesExt;
use crate::map::ErasedTy;
use crate::opt::Opt;
use crate::ser::Services;
use crate::set::SetOpt;
use crate::Error;
use crate::RawVal;
use crate::Uid;

/// The [`Store`] processer save the value of given option into
/// [`ValServices`](crate::ser::ValService) and [`RawValServices`](crate::ser::RawValService).
pub trait Store<Set, Value> {
    type Ret;
    type Error: Into<Error>;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error>;
}

impl<Func, Set, Value, Ret, Err> Store<Set, Value> for Func
where
    Err: Into<Error>,
    Func: FnMut(
            Uid,
            &mut Set,
            &mut Services,
            Option<&RawVal>,
            Option<Value>,
        ) -> Result<Option<Ret>, Err>
        + Send
        + Sync,
{
    type Ret = Ret;
    type Error = Err;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        (self)(uid, set, ser, raw, val)
    }
}

/// Null store, do nothing. See [`Action`](crate::opt::Action) for default store.
pub struct NullStore;

impl<Set, Value> Store<Set, Value> for NullStore {
    type Ret = Value;

    type Error = Error;

    fn process(
        &mut self,
        _: Uid,
        _: &mut Set,
        _: &mut Services,
        _: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        Ok(val)
    }
}

/// Vector store, append the value to the [`ValService`](crate::ser::ValService)
/// if option's action is Action::App.
/// See [`Action`](crate::opt::Action) for default store.
pub struct VecStore;

impl<Set, Value: ErasedTy> Store<Set, Vec<Value>> for VecStore
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
{
    type Ret = ();

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Vec<Value>>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let has_value = val.is_some();

        // Set the value if return Some(Value)
        if let Some(val) = val {
            let raw_ser = ser.ser_rawval_mut()?;

            if let Some(raw) = raw {
                raw_ser.push(uid, raw.clone());
            }

            let val_ser = ser.ser_val_mut()?;

            if let Some(opt) = set.get(uid) {
                if opt.action().is_app() {
                    for value in val {
                        val_ser.push(uid, value);
                    }
                }
            }
        }

        Ok(has_value.then_some(()))
    }
}
