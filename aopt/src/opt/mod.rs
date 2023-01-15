pub(crate) mod action;
pub(crate) mod aopt;
pub(crate) mod config;
pub(crate) mod creator;
pub(crate) mod help;
pub(crate) mod index;
pub(crate) mod infer;
pub(crate) mod info;
pub(crate) mod parser;
#[cfg(feature = "serde")]
pub(crate) mod serde;
pub(crate) mod style;
pub(crate) mod value;

pub use self::action::Action;
pub use self::aopt::AOpt;
pub use self::config::Config;
pub use self::config::ConfigValue;
pub use self::config::OptConfig;
pub use self::creator::Creator;
pub use self::help::Help;
pub use self::index::Index;
pub use self::infer::Infer;
pub use self::info::ConstrctInfo;
pub use self::info::Information;
pub use self::parser::StrParser;
#[cfg(feature = "serde")]
pub use self::serde::Deserialize;
#[cfg(feature = "serde")]
pub use self::serde::Serde;
#[cfg(feature = "serde")]
pub use self::serde::Serialize;
pub use self::style::Style;
pub use self::value::OptValueExt;

use std::any::TypeId;
use std::fmt::Debug;

use crate::value::ValAccessor;
use crate::Error;
use crate::Str;
use crate::Uid;

pub const BOOL_TRUE: &str = "true";

pub const BOOL_FALSE: &str = "false";

/// Option parser using for parsing option constructor string.
pub trait OptParser {
    type Output;
    type Error: Into<Error>;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error>;
}

pub trait Opt: Debug {
    fn reset(&mut self);

    fn uid(&self) -> Uid;

    /// The name of option.
    fn name(&self) -> &Str;

    fn r#type(&self) -> &TypeId;

    fn hint(&self) -> &Str;

    fn help(&self) -> &Str;

    fn valid(&self) -> bool;

    fn matched(&self) -> bool;

    /// If the option is optional.
    fn force(&self) -> bool;

    fn action(&self) -> &Action;

    /// The index of option.
    fn idx(&self) -> Option<&Index>;

    /// The alias the option.
    fn alias(&self) -> Option<&Vec<Str>>;

    fn accessor(&self) -> &ValAccessor;

    fn accessor_mut(&mut self) -> &mut ValAccessor;

    fn set_uid(&mut self, uid: Uid);

    fn set_matched(&mut self, matched: bool);

    fn mat_style(&self, style: Style) -> bool;

    fn mat_force(&self, force: bool) -> bool;

    fn mat_name(&self, name: Option<&Str>) -> bool;

    fn mat_alias(&self, name: &Str) -> bool;

    fn mat_idx(&self, index: Option<(usize, usize)>) -> bool;

    fn init(&mut self) -> Result<(), Error>;
}
