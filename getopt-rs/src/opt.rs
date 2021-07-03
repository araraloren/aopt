pub mod callback;
pub mod help;
pub mod index;
pub mod parser;
pub mod style;
pub mod value;

// options mod
pub mod bool;
pub mod str;
pub mod array;
pub mod int;
pub mod uint;
pub mod flt;
pub mod pos;
pub mod cmd;
pub mod main;
pub mod example;

use std::fmt::Debug;

use crate::err::{Error, Result};
use crate::opt::callback::CallbackType;
use crate::opt::help::HelpInfo;
use crate::opt::index::Index as OptIndex;
use crate::opt::value::Value as OptValue;
use crate::uid::Uid;

pub trait Type {
    fn get_type_name(&self) -> &'static str;

    fn is_deactivate_style(&self) -> bool {
        false
    }

    fn match_style(&self, style: style::Style) -> bool;

    fn check(&self) -> Result<bool>;

    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait Identifier {
    fn get_uid(&self) -> Uid;

    fn set_uid(&mut self, uid: Uid);
}

pub trait Callback {
    fn get_callback_type(&self) -> &CallbackType;

    fn set_callback_type(&mut self, callback_type: CallbackType);

    fn is_need_invoke(&self) -> bool;

    fn set_invoke(&mut self, invoke: bool, mutbale: bool);

    fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool;
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
    fn get_alias(&self) -> Option<&Vec<(String, String)>>;

    fn add_alias(&mut self, prefix: String, name: String);

    fn rem_alias(&mut self, prefix: &str, name: &str);

    fn match_alias(&self, prefix: &str, name: &str) -> bool;
}

pub trait Optional {
    fn get_optional(&self) -> bool;

    fn set_optional(&mut self, optional: bool);

    fn match_optional(&self, optional: bool) -> bool;
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
    fn get_index(&self) -> Option<&OptIndex>;

    fn set_index(&mut self, index: OptIndex);

    fn match_index(&self, total: u64, current: u64) -> bool;
}

pub trait Help {
    fn set_hint(&mut self, hint: String);

    fn set_help(&mut self, help: String);

    fn get_hint(&self) -> &str {
        self.get_help_info().get_hint().as_str()
    }

    fn get_help(&self) -> &str {
        self.get_help_info().get_help().as_str()
    }

    fn get_help_info(&self) -> &HelpInfo;
}

pub trait Opt:
    Type + Identifier + Name + Callback + Alias + Optional + Value + Index + Help + Debug
{
}

pub trait NonOpt: Opt { }