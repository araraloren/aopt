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
use crate::AStr;
use crate::Error;
use crate::Uid;

pub const BOOL_TRUE: &str = "true";

pub const BOOL_FALSE: &str = "false";

/// Cmd represents a sub command flag wrapped the `bool` option, it is force required in default.
///
/// See [`cmd_check`](crate::set::SetChecker::cmd_check) of
/// [`DefaultSetChecker`](crate::parser::DefaultSetChecker) for default checking behavior.
///
/// # Example
///
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::opt::Cmd;
/// #
/// # fn main() -> Result<(), aopt::Error> {
///     
/// let mut parser = AFwdParser::default();
///
/// // `Cmd` has a default position `@1`.
/// parser.add_opt_i::<Cmd>("list: Set the list sub command")?;
/// parser.parse(ARef::new(Args::from(["app", "list"])))?;
///
/// // Get the value by `Infer::Val` type of `bool`.
/// assert_eq!(parser.find_val::<bool>("list")?, &true);
///
/// # Ok(())
/// # }
/// ```
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

/// Pos is a position option wrapper, it is matching based on position.
///
/// See [`pos_check`](crate::set::SetChecker::pos_check) of
/// [`DefaultSetChecker`](crate::parser::DefaultSetChecker) for default checking behavior.
///
/// # Example
///
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::opt::Pos;
/// #
/// # fn main() -> Result<(), aopt::Error> {
///     
/// let mut parser = AFwdParser::default();
///
/// // Name is not important.
/// parser.add_opt_i::<Pos<String>>("pos_accept_string@1: Set the string value")?;
///
/// parser.parse(ARef::new(Args::from(["app", "value"])))?;
///
/// // Get the value by `Infer::Val` type of `String`.
/// assert_eq!(parser.find_val::<String>("pos_accept_string")?, &String::from("value"));
///
/// # Ok(())
/// # }
/// ```
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

/// Main are always matched; it is using for running logical before [`Policy`](crate::parser::Policy) ending.
///
/// See [`post_check`](crate::set::SetChecker::post_check) of
/// [`DefaultSetChecker`](crate::parser::DefaultSetChecker) for default checking behavior.
/// # Example
///
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::opt::Main;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), aopt::Error> {
///     
/// let mut parser = AFwdParser::default();
///
/// // `Main` has a default position `@*`.
/// parser.add_opt_i::<Main>("main_function: Call the main function")?
///       // Main do nothing in default, you must change the `Action` if you want save value
///       .set_action(Action::Set)
///       .on(|_: &mut ASet, _: &mut ASer, val: ctx::RawVal|{
///             assert_eq!(val.deref(), &RawVal::from("app"));
///             Ok(Some(String::from("main_function called")))
///       })?;
///
/// parser.parse(ARef::new(Args::from(["app", "list"])))?;
///
/// // Get the value of main function returned.
/// assert_eq!(parser.find_val::<String>("main_function")?, &String::from("main_function called"));
///
/// # Ok(())
/// # }
/// ```
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

/// Simple option type wrapper, implemented [`Infer`](crate::value::Infer).
/// It works with the types are implemented [`RawValParser`](crate::value::RawValParser).
///
/// # Example
///
/// ```rust
/// # use aopt::Error;
/// # use aopt::value::raw2str;
/// # use aopt::prelude::*;
/// # use std::ops::Deref;
///
/// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// pub struct Name(String);
///
/// impl RawValParser for Name {
///     type Error = Error;
///
///     fn parse(arg: Option<&RawVal>, _: &Ctx) -> Result<Self, Self::Error> {
///         Ok(Name(raw2str(arg)?.to_owned()))
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut parser = AFwdParser::default();
///
/// // add the option wrap with `MutOpt`
/// parser.add_opt_i::<MutOpt<Name>>("-e: Set the name")?;
///
/// parser.parse(ARef::new(Args::from(["app", "-e=foo"])))?;
///
/// // Get the value through value type `Name`
/// assert_eq!(parser.find_val::<Name>("-e")?, &Name("foo".to_owned()));
///
/// #    Ok(())
/// # }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MutOpt<T>(pub T);

impl<T> MutOpt<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for MutOpt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MutOpt<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Simple option type wrapper, implemented [`Infer`](crate::value::Infer).
/// It works with the types are implemented [`RawValParser`](crate::value::RawValParser).
///
/// # Example
///
/// ```rust
/// # use aopt::Error;
/// # use aopt::value::raw2str;
/// # use aopt::prelude::*;
/// # use std::ops::Deref;
///
/// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// pub struct Name(String);
///
/// impl RawValParser for Name {
///     type Error = Error;
///
///     fn parse(arg: Option<&RawVal>, _: &Ctx) -> Result<Self, Self::Error> {
///         Ok(Name(raw2str(arg)?.to_owned()))
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut parser = AFwdParser::default();
///
/// // add the option wrap with `RefOpt`
/// parser.add_opt_i::<RefOpt<'_, Name>>("-e: Set the name")?;
///
/// parser.parse(ARef::new(Args::from(["app", "-e=foo"])))?;
///
/// // Get the value through value type `Name`
/// assert_eq!(parser.find_val::<Name>("-e")?, &Name("foo".to_owned()));
///
/// #    Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefOpt<'a, T>(pub &'a T);

impl<'a, T> RefOpt<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self(value)
    }
}

impl<'a, T> Deref for RefOpt<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
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

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error>;
}

pub trait Opt: Debug {
    fn reset(&mut self);

    fn uid(&self) -> Uid;

    /// The name of option.
    fn name(&self) -> &AStr;

    /// The type of option.
    fn r#type(&self) -> &TypeId;

    /// The help hint of option such as `--flag`.
    fn hint(&self) -> &AStr;

    /// The help message of option.
    fn help(&self) -> &AStr;

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
    fn alias(&self) -> Option<&Vec<AStr>>;

    fn accessor(&self) -> &ValAccessor;

    fn accessor_mut(&mut self) -> &mut ValAccessor;

    fn ignore_alias(&self) -> bool;

    fn ignore_name(&self) -> bool;

    fn ignore_index(&self) -> bool;

    fn set_uid(&mut self, uid: Uid);

    fn set_matched(&mut self, matched: bool);

    fn mat_style(&self, style: Style) -> bool;

    fn mat_force(&self, force: bool) -> bool;

    fn mat_name(&self, name: Option<&AStr>) -> bool;

    fn mat_alias(&self, name: &AStr) -> bool;

    fn mat_index(&self, index: Option<(usize, usize)>) -> bool;

    fn init(&mut self) -> Result<(), Error>;
}
