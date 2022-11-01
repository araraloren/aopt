use std::marker::PhantomData;

use crate::ctx::Store;
use crate::opt::Opt;
use crate::opt::ValAction;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::ValService;
use crate::set::SetExt;
use crate::Error;
use crate::RawVal;
use crate::Uid;

pub struct ValStore<Ret>(PhantomData<Ret>);

impl<Ret: 'static> ValStore<Ret> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

impl<Set, Value, Ret> Store<Set, Value> for ValStore<Ret>
where
    Value: 'static,
    Set::Opt: Opt,
    Ret: Default,
    Set: crate::set::Set,
{
    type Ret = Ret;

    type Error = Error;

    fn process(
        &mut self,
        uid: Uid,
        set: &mut Set,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let has_value = val.is_some();

        // Set the value if return Some(Value)
        if let Some(val) = val {
            let raw_ser = ser.service_mut::<RawValService<RawVal>>()?;

            if let Some(raw) = raw {
                raw_ser.push(uid, raw.clone());
            }

            let action = set.opt(uid)?.action();
            let val_ser = ser.service_mut::<ValService>()?;

            match action {
                ValAction::Set => {
                    val_ser.set(uid, vec![val]);
                }
                ValAction::App => {
                    val_ser.push(uid, val);
                }
                ValAction::Pop => {
                    val_ser.pop::<Value>(uid);
                }
                ValAction::Cnt => {
                    val_ser.entry::<u64>(uid).or_insert(vec![0])[0] += 1;
                }
                ValAction::Null => {
                    //DO NOTHING
                }
            }
        }

        Ok(has_value.then(|| Ret::default()))
    }
}
