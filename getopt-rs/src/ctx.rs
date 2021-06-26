
use std::fmt::Debug;

use crate::opt::{Opt, style::Style};
use crate::err::{Result, Error};

pub trait Context: Debug {
    fn match_opt(&self, opt: &dyn Opt) -> bool;

    fn process_opt(&mut self, opt: &mut dyn Opt) -> Result<bool>;

    fn get_matched_index(&self) -> Option<usize>;

    fn get_style(&self) -> Style;

    fn get_next_argument(&self) -> &Option<String>;

    fn is_need_argument(&self) -> bool;

    fn is_matched(&self) -> bool {
        self.get_matched_index().is_some()
    }
}