mod check;
mod delay_parser;
mod pre_parser;
mod simple_parser;
mod state;
pub(crate) mod testutil;

use std::cell::RefCell;
use std::fmt::Debug;
use ustr::Ustr;

use crate::arg::Argument;
use crate::err::Result;
use crate::opt::{OptCallback, OptValue};
use crate::set::Set;
use crate::uid::Uid;

pub(crate) use std::collections::hash_map::Iter as HashMapIter;

pub use check::default_nonopt_check;
pub use check::default_opt_check;
pub use check::default_post_check;
pub use check::default_pre_check;
pub use delay_parser::DelayParser;
pub use pre_parser::PreParser;
pub use simple_parser::SimpleParser;
pub use state::ParserState;

#[derive(Debug)]
pub struct OptValueKeeper {
    noa_index: usize,

    value: OptValue,
}

pub trait Parser: Debug {
    fn parse(
        &mut self,
        set: &mut dyn Set,
        iter: &mut dyn Iterator<Item = Argument>,
    ) -> Result<bool>;

    fn invoke_callback(
        &self,
        uid: Uid,
        set: &mut dyn Set,
        noa_index: usize,
        value: OptValue,
    ) -> Result<Option<OptValue>>;

    fn pre_check(&self, set: &dyn Set) -> Result<bool>
    where
        Self: Sized,
    {
        check::default_pre_check(set, self)
    }

    fn check_opt(&self, set: &dyn Set) -> Result<bool>
    where
        Self: Sized,
    {
        check::default_opt_check(set, self)
    }

    fn check_nonopt(&self, set: &dyn Set) -> Result<bool>
    where
        Self: Sized,
    {
        check::default_nonopt_check(set, self)
    }

    fn post_check(&self, set: &dyn Set) -> Result<bool>
    where
        Self: Sized,
    {
        check::default_post_check(set, self)
    }

    fn add_callback(&mut self, uid: Uid, callback: OptCallback);

    fn get_callback(&self, uid: Uid) -> Option<&RefCell<OptCallback>>;

    fn callback_iter(&self) -> HashMapIter<'_, Uid, RefCell<OptCallback>>;

    fn get_noa(&self) -> &[Ustr];

    fn reset(&mut self);
}
