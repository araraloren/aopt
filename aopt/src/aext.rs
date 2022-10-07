use crate::ctx::ExtractFromCtx;
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

    fn noa_service(&self) -> &NOAService;

    fn noa_service_mut(&mut self) -> &mut NOAService;

    fn data_service(&self) -> &DataService;

    fn data_service_mut(&mut self) -> &mut DataService;

    fn value_service(&self) -> &ValueService<Value>;

    fn value_service_mut(&mut self) -> &mut ValueService<Value>;

    fn invoke_service(&self) -> &InvokeService<Set, Value>;

    fn invoke_service_mut(&mut self) -> &mut InvokeService<Set, Value>;

    fn check_service(&self) -> &CheckService<Set, Value>;

    fn check_service_mut(&mut self) -> &mut CheckService<Set, Value>;

    fn get_data<T>(&self) -> Option<&T>
    where
        T: 'static;

    fn get_data_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static;

    fn ins_data<T>(&mut self, value: T) -> Option<T>
    where
        T: 'static;

    fn rem_data<T>(&mut self) -> Option<T>
    where
        T: 'static;

    fn get_val(&self, uid: Uid) -> Option<&Value>;

    fn get_vals(&self, uid: Uid) -> Option<&Vec<Value>>;

    fn get_val_mut(&mut self, uid: Uid) -> Option<&mut Value>;

    fn get_vals_mut(&mut self, uid: Uid) -> Option<&mut Vec<Value>>;

    fn reg_callback<H, Args>(&mut self, uid: Uid, handler: H) -> &mut Self
    where
        Args: ExtractFromCtx<Set, Error = Error> + 'static,
        H: Handler<Set, Args, Output = Option<Value>, Error = Error> + 'static;
}
