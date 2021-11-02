pub mod callback;
pub mod help;
pub mod index;
pub mod nonopt;
pub mod opt;
pub mod parser;
pub mod style;
pub mod value;

use std::fmt::Debug;

use crate::err::Result;
use crate::uid::Uid;
use crate::OptStr;

pub use self::callback::{Callback as OptCallback, CallbackType};
pub use self::help::HelpInfo;
pub use self::index::Index as OptIndex;
pub use self::nonopt::{cmd::CmdCreator, main::MainCreator, pos::PosCreator};
pub use self::opt::{
    array::ArrayCreator, bool::BoolCreator, flt::FltCreator, int::IntCreator, str::StrCreator,
    uint::UintCreator,
};
pub use self::parser::{parse_option_str, DataKeeper};
pub use self::style::Style;
pub use self::value::Value as OptValue;

pub trait Type {
    fn get_type_name(&self) -> &'static str;

    fn is_deactivate_style(&self) -> bool {
        false
    }

    fn match_style(&self, style: style::Style) -> bool;

    fn check(&self) -> Result<()>;

    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait Identifier {
    fn get_uid(&self) -> Uid;

    fn set_uid(&mut self, uid: Uid);
}

pub trait Callback {
    fn is_need_invoke(&self) -> bool;

    fn set_invoke(&mut self, invoke: bool);

    fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool;

    fn set_callback_ret(&mut self, ret: Option<OptValue>) -> Result<()>;
}

pub trait Name {
    fn get_name(&self) -> OptStr;

    fn get_prefix(&self) -> OptStr;

    fn set_name(&mut self, string: OptStr);

    fn set_prefix(&mut self, string: OptStr);

    fn match_name(&self, name: OptStr) -> bool;

    fn match_prefix(&self, prefix: OptStr) -> bool;
}

pub trait Alias {
    fn get_alias(&self) -> Option<&Vec<(OptStr, OptStr)>>;

    fn add_alias(&mut self, prefix: OptStr, name: OptStr);

    fn rem_alias(&mut self, prefix: OptStr, name: OptStr);

    fn match_alias(&self, prefix: OptStr, name: OptStr) -> bool;
}

pub trait Optional {
    fn get_optional(&self) -> bool;

    fn set_optional(&mut self, optional: bool);

    fn match_optional(&self, optional: bool) -> bool;
}

pub trait Value {
    fn get_value(&self) -> &OptValue;

    fn get_value_mut(&mut self) -> &mut OptValue;

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
