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

    fn undo(&mut self, opt: &mut dyn Opt);

    fn get_value(&self) -> Option<&OptValue>;

    fn take_value(&mut self) -> Option<OptValue>;

    fn set_value(&mut self, value: OptValue);

    fn get_matched_uid(&self) -> Option<Uid> {
        None
    }

    fn set_matched_uid(&mut self, _uid: Option<Uid>) {}

    fn get_matched_index(&self) -> Option<usize> {
        None
    }

    fn set_matched_index(&mut self, _index: Option<usize>) {}

    fn get_style(&self) -> Style;

    fn get_next_argument(&self) -> &Option<Ustr>;

    fn is_comsume_argument(&self) -> bool;

    fn is_matched(&self) -> bool {
        self.get_matched_uid().is_some()
    }
}
