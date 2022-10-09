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

use crate::ctx::Ctx;
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
    fn name(&self) -> &Str;

    fn set_name(&mut self, name: Str);

    fn mat_name(&self, name: &Str) -> bool;
}

pub trait Prefix {
    fn pre(&self) -> Option<&Str>;

    fn set_pre(&mut self, prefix: Option<Str>);

    fn mat_pre(&self, prefix: Option<&Str>) -> bool;
}

pub trait Optional {
    fn opt(&self) -> bool;

    fn set_opt(&mut self, optional: bool);

    fn mat_opt(&self, optional: bool) -> bool;
}

pub trait Alias {
    fn alias(&self) -> Option<&Vec<(Str, Str)>>;

    fn add_alias(&mut self, prefix: Str, name: Str);

    fn rem_alias(&mut self, prefix: &Str, name: &Str);

    fn mat_alias(&self, prefix: &Str, name: &Str) -> bool;
}

pub trait Index {
    fn idx(&self) -> Option<&OptIndex>;

    fn set_idx(&mut self, index: Option<OptIndex>);

    fn mat_idx(&self, index: Option<(usize, usize)>) -> bool;
}

pub trait Help {
    fn hint(&self) -> &Str;

    fn help(&self) -> &Str;

    fn set_hint(&mut self, hint: Str);

    fn set_help(&mut self, help: Str);
}

pub trait Opt: Name + Help + Alias + Index + Prefix + Optional + Debug {
    fn reset(&mut self);

    fn check(&self) -> bool;

    fn uid(&self) -> Uid;

    fn set_uid(&mut self, uid: Uid);

    fn setted(&self) -> bool;

    fn ty(&self) -> Str;

    fn is_deact(&self) -> bool;

    fn mat_sty(&self, style: OptStyle) -> bool;

    fn has_callback(&self) -> bool;

    fn set_setted(&mut self, setted: bool);

    fn invoke(&mut self, ser: &mut Services, ctx: &Ctx) -> Result<Option<Str>, Error>;

    fn val(
        &mut self,
        arg: Option<&Str>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error>;
}
