use crate::ctx::ExtractCtx;
use crate::ctx::Handler;
use crate::opt::OptParser;
use crate::ser::CheckService;
use crate::ser::DataService;
use crate::ser::InvokeService;
use crate::ser::NOAService;
use crate::ser::ValueService;
use crate::Error;
use crate::Uid;

pub trait ASetExt {
    fn new_set() -> Self;
}

pub trait ASetConfigExt {
    fn with_default_prefix(self) -> Self;

    fn with_default_creator(self) -> Self;
}

pub trait APolicyExt<Set: 'static, Value: 'static> {
    fn new_set<T>() -> T
    where
        T: ASetExt + crate::set::Set + OptParser;

    fn new_services<T>() -> T
    where
        T: AServiceExt<Set, Value>;
}

pub trait AServiceExt<Set: 'static, Value: 'static> {
    fn new_services() -> Self;

    fn noa_ser(&self) -> &NOAService;

    fn noa_ser_mut(&mut self) -> &mut NOAService;

    fn data_ser(&self) -> &DataService;

    fn data_ser_mut(&mut self) -> &mut DataService;

    fn val_ser(&self) -> &ValueService<Value>;

    fn val_ser_mut(&mut self) -> &mut ValueService<Value>;

    fn invoke_ser(&self) -> &InvokeService<Set, Value>;

    fn invoke_ser_mut(&mut self) -> &mut InvokeService<Set, Value>;

    fn check_ser(&self) -> &CheckService<Set, Value>;

    fn check_ser_mut(&mut self) -> &mut CheckService<Set, Value>;

    fn data<T>(&self) -> Option<&T>
    where
        T: 'static;

    fn data_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static;

    fn ins_data<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static;

    fn rem_data<T>(&mut self) -> Option<T>
    where
        T: 'static;

    fn val(&self, uid: Uid) -> Option<&Value>;

    fn vals(&self, uid: Uid) -> Option<&Vec<Value>>;

    fn val_mut(&mut self, uid: Uid) -> Option<&mut Value>;

    fn vals_mut(&mut self, uid: Uid) -> Option<&mut Vec<Value>>;

    fn reg_callback<H, Args>(&mut self, uid: Uid, handler: H) -> &mut Self
    where
        Args: ExtractCtx<Set, Error = Error> + 'static,
        H: Handler<Set, Args, Output = Value, Error = Error> + 'static;
}
