pub mod check;
pub mod simple;
pub mod state;
pub(crate) mod testutil;

use std::cell::RefCell;
use std::fmt::Debug;

use crate::err::Result;
use crate::opt::{OptCallback, OptValue};
use crate::set::Set;
use crate::uid::Uid;
pub(crate) use std::collections::hash_map::Iter as HashMapIter;

pub use simple::SimpleParser;
pub use state::ParserState;

pub trait Parser<S>: Debug
where
    Self: Sized,
    S: Set,
{
    fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>>;

    fn invoke_callback(
        &self,
        uid: Uid,
        set: &mut S,
        noa_index: usize,
        value: OptValue,
    ) -> Result<Option<OptValue>>;

    fn pre_check(&self, set: &S) -> Result<bool> {
        check::default_pre_check(set, self)
    }

    fn check_opt(&self, set: &S) -> Result<bool> {
        check::default_opt_check(set, self)
    }

    fn check_nonopt(&self, set: &S) -> Result<bool> {
        check::default_nonopt_check(set, self)
    }

    fn post_check(&self, set: &S) -> Result<bool> {
        check::default_post_check(set, self)
    }

    fn add_callback(&mut self, uid: Uid, callback: OptCallback);

    fn get_callback(&self, uid: Uid) -> Option<&RefCell<OptCallback>>;

    fn callback_iter(&self) -> HashMapIter<'_, Uid, RefCell<OptCallback>>;

    fn reset(&mut self);
}

#[derive(Debug)]
pub struct ReturnValue<'a, S: Set> {
    pub noa: &'a Vec<String>,
    pub set: S,
}
