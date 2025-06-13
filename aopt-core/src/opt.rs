pub(crate) mod action;
pub(crate) mod index;
pub(crate) mod style;

use std::any::TypeId;
use std::fmt::Debug;

use crate::value::ValAccessor;
use crate::Error;
use crate::Uid;

pub use self::action::Action;
pub use self::index::Index;
pub use self::style::Style;

pub const BOOL_TRUE: &str = "true";

pub const BOOL_FALSE: &str = "false";

pub trait Opt: Debug {
    fn reset(&mut self);

    /// The Uid of option.
    fn uid(&self) -> Uid;

    /// The name of option.
    fn name(&self) -> &str;

    /// The type of option.
    fn r#type(&self) -> &TypeId;

    /// The help hint of option such as `--flag`.
    fn hint(&self) -> &str;

    /// The help message of option.
    fn help(&self) -> &str;

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
    fn alias(&self) -> Option<&Vec<String>>;

    fn accessor(&self) -> &ValAccessor;

    fn accessor_mut(&mut self) -> &mut ValAccessor;

    fn ignore_alias(&self) -> bool;

    fn ignore_name(&self) -> bool;

    fn ignore_index(&self) -> bool;

    fn set_uid(&mut self, uid: Uid);

    fn set_matched(&mut self, matched: bool);

    fn mat_style(&self, style: Style) -> bool;

    fn mat_force(&self, force: bool) -> bool;

    fn mat_name(&self, name: Option<&str>) -> bool;

    fn mat_alias(&self, name: &str) -> bool;

    fn mat_index(&self, index: Option<(usize, usize)>) -> bool;

    fn init(&mut self) -> Result<(), Error>;

    fn set_name(&mut self, name: impl Into<String>) -> &mut Self;

    fn set_type(&mut self, r#type: TypeId) -> &mut Self;

    fn set_value(&mut self, value: ValAccessor) -> &mut Self;

    fn set_hint(&mut self, hint: impl Into<String>) -> &mut Self;

    fn set_help(&mut self, help: impl Into<String>) -> &mut Self;

    fn set_action(&mut self, action: Action) -> &mut Self;

    fn set_style(&mut self, styles: Vec<Style>) -> &mut Self;

    fn set_index(&mut self, index: Option<Index>) -> &mut Self;

    fn set_force(&mut self, force: bool) -> &mut Self;

    fn add_alias(&mut self, name: impl Into<String>) -> &mut Self;

    fn rem_alias(&mut self, name: &str) -> &mut Self;

    fn set_ignore_name(&mut self, ignore_name: bool) -> &mut Self;

    fn set_ignore_alias(&mut self, ignore_alias: bool) -> &mut Self;

    fn set_ignore_index(&mut self, ignore_index: bool) -> &mut Self;
}
