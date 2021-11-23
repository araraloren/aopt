pub mod nonopt;
pub mod opt;

use std::fmt::Debug;

use crate::ctx::Context;
use crate::err::Result;
use crate::set::Set;
use crate::uid::Uid;

pub use nonopt::NonOptMatcher;
pub use opt::OptMatcher;

pub trait Info: Debug {
    fn info_uid(&self) -> Uid;
}

pub trait Proc<M: Matcher>: Debug {
    fn process(&mut self, matcher: &mut M, set: &mut dyn Set) -> Result<bool>;
}

pub trait Matcher: Debug {
    fn uid(&self) -> Uid;

    fn add_ctx(&mut self, ctx: Box<dyn Context>);

    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>>;

    fn get_ctx_mut(&mut self, index: usize) -> Option<&mut Box<dyn Context>>;

    fn process(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<&mut Box<dyn Context>>>;

    fn undo(&mut self);

    fn is_matched(&self) -> bool;

    fn is_comsume_argument(&self) -> bool;

    fn quit(&self) -> bool;

    fn reset(&mut self);

    fn len(&self) -> usize;
}
