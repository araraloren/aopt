use std::fmt::Debug;

pub mod nonopt;
pub mod opt;

use crate::err::Result;
use crate::opt::{Opt, OptValue, Style};
use crate::Ustr;

pub use self::nonopt::NonOptContext;
pub use self::opt::OptContext;

pub trait Context: Debug {
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool>;

    fn get_value(&self) -> Option<&OptValue>;

    fn take_value(&mut self) -> Option<OptValue>;

    fn set_value(&mut self, value: OptValue);

    fn get_matched_index(&self) -> Option<usize>;

    fn set_matched_index(&mut self, index: Option<usize>);

    fn get_style(&self) -> Style;

    fn get_next_argument(&self) -> &Option<Ustr>;

    fn is_comsume_argument(&self) -> bool;

    fn is_matched(&self) -> bool {
        self.get_matched_index().is_some()
    }
}
