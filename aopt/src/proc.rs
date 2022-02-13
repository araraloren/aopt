mod nonopt;
mod opt;

use std::fmt::Debug;

use crate::ctx::Context;
use crate::err::Result;
use crate::opt::Style;
use crate::parser::ValueKeeper;
use crate::set::Set;
use crate::uid::Uid;

pub use nonopt::NonOptMatcher;
pub use opt::OptMatcher;

pub trait Info: Debug {
    fn info_uid(&self) -> Uid;
}

/// Trait using for process [`Matcher`].
pub trait Proc<M: Matcher>: Debug {
    fn process<S: Set>(
        &mut self,
        matcher: &mut M,
        set: &mut S,
        invoke: bool,
    ) -> Result<Vec<ValueKeeper>>;
}

/// Trait using for hold [`Context`], and matched with [`Opt`](crate::opt::Opt) in [`Set`].
pub trait Matcher: Debug {
    fn get_uid(&self) -> Uid;

    /// Add [`Context`] to current [`Matcher`].
    fn add_ctx(&mut self, ctx: Box<dyn Context>);

    /// Get [`Context`] reference with index.
    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>>;

    /// Get mutable [`Context`] reference with index.
    fn get_ctx_mut(&mut self, index: usize) -> Option<&mut Box<dyn Context>>;

    /// Get [`Style`] current [`Matcher] support.
    fn get_style(&self) -> Style;

    /// Matching specify [`Opt`](crate::opt::Opt) with current [`Matcher`].
    ///
    /// # Return
    ///
    /// Return the [`Context`] that matched successful. Or return [`None`] if not matched.
    fn process<S: Set>(&mut self, uid: Uid, set: &mut S) -> Result<Option<&mut Box<dyn Context>>>;

    /// Revert the change applied by [`process`](Matcher::process).
    fn undo<S: Set>(&mut self, set: &mut S);

    /// If all the [`Context`] matched.
    fn is_matched(&self) -> bool;

    /// Return True if the [`Matcher`] consumed command line argument.
    fn is_comsume_argument(&self) -> bool;

    /// Return True if we can quit midway from matching.
    fn quit(&self) -> bool;

    /// Reset the [`Matcher`].
    fn reset(&mut self);

    /// Return the count of [`Context`] in current [`Matcher`].
    fn len(&self) -> usize;
}
