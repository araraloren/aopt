use crate::opt::Opt;
use crate::ser::ServicesExt;
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
    ) -> Result<Option<Self::Ret>, Self::Error>;
}

impl<Func, Set, Ser, Value, Ret, Err> Store<Set, Ser, Value> for Func
where
    Err: Into<Error>,
    Func:
        FnMut(Uid, &mut Set, &mut Ser, Option<&RawVal>, Option<Value>) -> Result<Option<Ret>, Err>,
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
    ) -> Result<Option<Self::Ret>, Self::Error> {
        (self)(uid, set, ser, raw, val)
    }
}

/// Null store, do nothing. See [`Action`](crate::opt::Action) for default store.
pub struct NullStore;

impl<Set, Ser, Value> Store<Set, Ser, Value> for NullStore {
    type Ret = Value;

    type Error = Error;

    fn process(
        &mut self,
        _: Uid,
        _: &mut Set,
        _: &mut Ser,
        _: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        Ok(val)
    }
}

/// Vector store, append the value to the [`AnyValService`](crate::ser::AnyValService)
/// if option's action is Action::App.
/// See [`Action`](crate::opt::Action) for default store.
pub struct VecStore;

impl<Set, Ser, Value: 'static> Store<Set, Ser, Vec<Value>> for VecStore
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    Ser: ServicesExt,
{
    type Ret = ();

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Ser,
        raw: Option<&RawVal>,
        val: Option<Vec<Value>>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let has_value = val.is_some();

        // Set the value if return Some(Value)
        if let Some(val) = val {
            let raw_ser = ser.ser_rawval_mut();

            if let Some(raw) = raw {
                raw_ser.push(uid, raw.clone());
            }

            let val_ser = ser.ser_val_mut();

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
