pub(crate) mod match_noa;
pub(crate) mod match_opt;
pub(crate) mod proc_noa;
pub(crate) mod proc_opt;

use std::ffi::OsString;

pub use self::match_noa::NOAMatch;
pub use self::match_opt::OptMatch;
pub use self::proc_noa::NOAProcess;
pub use self::proc_opt::OptProcess;

use crate::opt::OptStyle;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::Uid;

/// [`Match`] match the configuration with [`Opt`](crate::opt::Opt).
pub trait Match {
    type Set: Set;
    type Error: Into<Error>;

    fn reset(&mut self);

    fn is_mat(&self) -> bool;

    fn mat_uid(&self) -> Option<Uid>;

    fn set_uid(&mut self, uid: Uid);

    fn sty(&self) -> OptStyle;

    fn arg(&self) -> Option<&Arc<OsString>>;

    fn consume(&self) -> bool;

    fn undo(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<(), Self::Error>;

    fn process(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<bool, Self::Error>;
}

/// [`Process`] matching the [`Opt`](crate::set::Set::Opt) with [`Match`], and return the first matched
/// [`Match`] if successful.
pub trait Process<M: Match> {
    type Set: Set;
    type Error: Into<Error>;

    fn reset(&mut self);

    fn quit(&self) -> bool;

    fn count(&self) -> usize;

    fn sty(&self) -> OptStyle;

    fn is_mat(&self) -> bool;

    fn consume(&self) -> bool;

    fn add_mat(&mut self, mat: M) -> &mut Self;

    fn mat(&self, index: usize) -> Option<&M>;

    fn mat_mut(&mut self, index: usize) -> Option<&mut M>;

    fn undo(&mut self, set: &mut Self::Set) -> Result<(), Self::Error>;

    fn process(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Option<usize>, Self::Error>;
}
