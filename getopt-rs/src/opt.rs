
pub mod help;
pub mod style;
pub mod value;
pub mod index;
pub mod callback;

use std::fmt::Debug;

use crate::err::Result;
use crate::uid::Uid;
use crate::opt::value::Value as OptValue;
use crate::opt::index::Index as OptIndex;
use crate::opt::help::HelpInfo;
use crate::opt::callback::CallbackType;

pub trait Type {
    fn get_type_name(&self) -> &'static str;

    fn is_deactivate_style(&self) -> bool;

    fn match_style(&self, style: style::Style) -> bool;

    fn check(&self) -> Result<bool>;
}

pub trait Identifier {
    fn get_uid(&self) -> Uid;

    fn set_uid(&mut self, uid: Uid);
}

pub trait Callback {
    fn get_callback_type(&self) -> &CallbackType;

    fn set_callback_type(&mut self, callback_type: CallbackType);

    fn is_need_invoke(&self) -> bool;

    fn set_invoke(&mut self, invoke: bool);
}

pub trait Name {
    fn get_name(&self) -> &str;

    fn get_prefix(&self) -> &str;

    fn set_name(&mut self, string: String);

    fn set_prefix(&mut self, string: String);

    fn match_name(&self, name: &str) -> bool;

    fn match_prefix(&self, prefix: &str) -> bool;
}

pub trait Alias {
    fn get_alias(&self) -> Option<Vec<(&str, &str)>>;

    fn add_alias(&mut self, prefix: String, name: String);

    fn rem_alias(&mut self, prefix: &str, name: &str);

    fn match_alias(&self, prefix: &str, name: &str);
}

pub trait Optional {
    fn get_optional(&self) -> bool;

    fn set_optional(&mut self, optional: bool);

    fn match_optional(&self, optional: bool);
}

pub trait Value {
    fn get_value(&self) -> &OptValue;

    fn get_default_value(&self) -> &OptValue;

    fn set_value(&mut self, value: OptValue);

    fn set_default_value(&mut self, value: OptValue);

    fn parse_value(&self, string: &str) -> Result<OptValue>;

    fn has_value(&self) -> bool;

    fn reset_value(&mut self);
}

pub trait Index {
    fn get_index(&self) -> &OptIndex;

    fn set_index(&mut self, index: OptIndex);

    fn match_index(&self, total: u64, current: u64) -> bool;
}

pub trait Help {
    fn set_hint(&mut self, hint: String);

    fn set_help(&mut self, help: String);

    fn get_help(&self) -> &HelpInfo;
}

pub trait Opt: Type +
               Identifier +
               Name +
               Callback +
               Alias +
               Optional +
               Value +
               Index +
               Help +
               Debug
            { }