pub(crate) mod action;
pub(crate) mod aopt;
pub(crate) mod config;
pub(crate) mod creator;
pub(crate) mod help;
pub(crate) mod index;
pub(crate) mod info;
pub(crate) mod parser;
#[cfg(feature = "serde")]
pub(crate) mod serialize;
pub(crate) mod style;
pub(crate) mod value;

pub use self::action::Action;
pub use self::aopt::AOpt;
pub use self::config::Config;
pub use self::config::ConfigValue;
pub use self::config::OptConfig;
pub use self::creator::BuiltInCtor;
pub use self::creator::Creator;
pub use self::help::Help;
pub use self::index::Index;
pub use self::info::ConstrctInfo;
pub use self::info::Information;
pub use self::parser::StrParser;
#[cfg(feature = "serde")]
pub use self::serialize::Deserialize;
#[cfg(feature = "serde")]
pub use self::serialize::Serde;
#[cfg(feature = "serde")]
pub use self::serialize::Serialize;
pub use self::style::Style;
pub use self::value::OptValueExt;

use std::any::TypeId;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::value::ValAccessor;
use crate::Error;
use crate::Str;
use crate::Uid;

pub const BOOL_TRUE: &str = "true";

pub const BOOL_FALSE: &str = "false";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cmd(pub bool);

impl Cmd {
    pub fn new(value: bool) -> Self {
        Self(value)
    }
}

impl Deref for Cmd {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cmd {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos<T = bool>(pub T);

impl<T> Pos<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Pos<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Pos<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Main<T = ()>(pub T);

impl<T> Main<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Main<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Main<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Any<T = ()>(pub T);

impl<T> Any<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Any<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Any<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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

    /// The type of option.
    fn r#type(&self) -> &TypeId;

    /// The help hint of option such as `--flag`.
    fn hint(&self) -> &Str;

    /// The help message of option.
    fn help(&self) -> &Str;

    fn valid(&self) -> bool;

    /// If the option matched.
    fn matched(&self) -> bool;

    /// If the option is force required.
    fn force(&self) -> bool;

    /// The associaed action of option.
    fn action(&self) -> &Action;

    /// The index of option.
    fn index(&self) -> Option<&Index>;

    /// The alias the option.
    fn alias(&self) -> Option<&Vec<Str>>;

    fn accessor(&self) -> &ValAccessor;

    fn accessor_mut(&mut self) -> &mut ValAccessor;

    fn ignore_alias(&self) -> bool;

    fn ignore_name(&self) -> bool;

    fn ignore_index(&self) -> bool;

    fn set_uid(&mut self, uid: Uid);

    fn set_matched(&mut self, matched: bool);

    fn mat_style(&self, style: Style) -> bool;

    fn mat_force(&self, force: bool) -> bool;

    fn mat_name(&self, name: Option<&Str>) -> bool;

    fn mat_alias(&self, name: &Str) -> bool;

    fn mat_index(&self, index: Option<(usize, usize)>) -> bool;

    fn init(&mut self) -> Result<(), Error>;
}
