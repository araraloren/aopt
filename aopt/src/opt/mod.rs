pub(crate) mod cb;
pub(crate) mod config;
pub(crate) mod creator;
pub(crate) mod help;
pub(crate) mod index;
pub(crate) mod info;
pub(crate) mod parser;
pub(crate) mod style;

pub use self::cb::OptCallback;
pub use self::config::Config;
pub use self::config::ConfigValue;
pub use self::config::OptConfig;
pub use self::creator::Creator;
pub use self::help::Help as OptHelp;
pub use self::index::Index as OptIndex;
pub use self::info::Information;
pub use self::info::OptConstrctInfo;
pub use self::parser::OptStringParser;
pub use self::style::Style as OptStyle;

use std::fmt::Debug;

use crate::ctx::Context;
use crate::prelude::Services;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Option parser using for parsing option constructor string.
pub trait OptParser {
    type Output;
    type Error: Into<Error>;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error>;
}

pub trait Name {
    fn get_name(&self) -> Str;

    fn set_name(&mut self, name: Str);

    fn match_name(&self, name: Str) -> bool;
}

pub trait Prefix {
    fn get_prefix(&self) -> Option<Str>;

    fn set_prefix(&mut self, prefix: Option<Str>);

    fn match_prefix(&self, prefix: Option<Str>) -> bool;
}

pub trait Optional {
    fn get_optional(&self) -> bool;

    fn set_optional(&mut self, optional: bool);

    fn match_optional(&self, optional: bool) -> bool;
}

pub trait Alias {
    fn get_alias(&self) -> Option<&Vec<(Str, Str)>>;

    fn add_alias(&mut self, prefix: Str, name: Str);

    fn rem_alias(&mut self, prefix: Str, name: Str);

    fn match_alias(&self, prefix: Str, name: Str) -> bool;
}

pub trait Index {
    fn get_index(&self) -> Option<&OptIndex>;

    fn set_index(&mut self, index: Option<OptIndex>);

    fn match_index(&self, index: Option<(usize, usize)>) -> bool;
}

pub trait Help {
    fn get_hint(&self) -> Str;

    fn get_help(&self) -> Str;

    fn set_hint(&mut self, hint: Str);

    fn set_help(&mut self, help: Str);
}

pub trait Opt: Name + Help + Alias + Index + Prefix + Optional + Debug {
    fn reset(&mut self);

    fn check(&self) -> bool;

    fn get_uid(&self) -> Uid;

    fn set_uid(&mut self, uid: Uid);

    fn set_setted(&mut self, setted: bool);

    fn get_setted(&self) -> bool;

    fn get_type_name(&self) -> Str;

    fn is_deactivate_style(&self) -> bool;

    fn match_style(&self, style: OptStyle) -> bool;

    fn has_callback(&self) -> bool;

    fn invoke_callback(&mut self, ser: &mut Services, ctx: Context) -> Result<Option<Str>, Error>;

    fn check_value(
        &mut self,
        arg: Option<Str>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error>;
}
