mod nonopt;
mod opt;

use std::fmt::Debug;
use ustr::Ustr;

use crate::err::Result;
use crate::opt::Opt;
use crate::opt::OptValue;
use crate::opt::Style;
use crate::uid::Uid;

pub use self::nonopt::NonOptContext;
pub use self::opt::OptContext;

/// The option matching context trait.
pub trait Context: Debug {
    /// Matching the option with current context.
    ///
    /// # Return
    ///
    /// - Ok(true)
    ///
    /// If the option matched.
    ///
    /// - Ok(false)
    ///
    /// If the option not matched.
    ///
    /// - Err
    ///
    /// If any error rasied when process the option.
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool>;

    /// Undo the matching operate.
    fn undo(&mut self, opt: &mut dyn Opt);

    /// Get the matching result value.
    fn get_value(&self) -> Option<&OptValue>;

    /// Take the ownership of result value.
    fn take_value(&mut self) -> Option<OptValue>;

    /// Set the matching result value.
    fn set_value(&mut self, value: OptValue);

    /// Get the matching uid.
    fn get_matched_uid(&self) -> Option<Uid> {
        None
    }

    /// Set the matching uid.
    fn set_matched_uid(&mut self, _uid: Option<Uid>) {}

    /// Get the matching non-option index.
    fn get_matched_index(&self) -> Option<usize> {
        None
    }

    /// Set the matching non-option index.
    fn set_matched_index(&mut self, _index: Option<usize>) {}

    /// Get current context support style.
    fn get_style(&self) -> Style;

    /// Get the argument of current context.
    fn get_argument(&self) -> &Option<Ustr>;

    /// Indicate if the context need consume command line item that treat as argument.
    fn is_comsume_argument(&self) -> bool;

    /// If the context matched any option.
    fn is_matched(&self) -> bool {
        self.get_matched_uid().is_some()
    }
}
