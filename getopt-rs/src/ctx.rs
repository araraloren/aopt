use std::fmt::Debug;

pub mod delay;
pub mod nonopt;
pub mod opt;

use crate::err::{Error, Result};
use crate::opt::{style::Style, Opt};
use crate::uid::Uid;

pub trait Context: Debug {
    fn get_uid(&self) -> Uid;

    fn match_opt(&self, opt: &dyn Opt) -> bool;

    fn process_opt(&mut self, opt: &mut dyn Opt) -> Result<bool>;

    fn get_matched_index(&self) -> Option<usize>;

    fn get_style(&self) -> Style;

    fn get_next_argument(&self) -> &Option<String>;

    fn is_comsume_argument(&self) -> bool;

    fn is_matched(&self) -> bool {
        self.get_matched_index().is_some()
    }
}
