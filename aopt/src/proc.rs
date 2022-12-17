pub(crate) mod noa;
pub(crate) mod opt;

pub use self::noa::NOAMatch;
pub use self::noa::NOAProcess;
pub use self::opt::OptMatch;
pub use self::opt::OptProcess;

use crate::opt::Style;
use crate::set::Ctor;
use crate::set::Set;
use crate::Error;
use crate::RawVal;
use crate::Uid;

/// [`Match`] match the configuration with [`Opt`](crate::opt::Opt).
pub trait Match {
    type Set: Set;
    type Error: Into<Error>;

    fn reset(&mut self);

    /// Return true if the [`Match`] matched.
    fn is_mat(&self) -> bool;

    /// Return the matched option `Uid`.
    fn mat_uid(&self) -> Option<Uid>;

    fn set_uid(&mut self, uid: Uid);

    /// Return the [`Style`] of option.
    fn style(&self) -> Style;

    /// Return the raw value.
    fn arg(&self) -> Option<&RawVal>;

    /// Return true if the option need cosume a argument.
    fn consume(&self) -> bool;

    fn undo(
        &mut self,
        opt: &mut <<Self::Set as Set>::Ctor as Ctor>::Opt,
    ) -> Result<(), Self::Error>;

    fn process(
        &mut self,
        opt: &mut <<Self::Set as Set>::Ctor as Ctor>::Opt,
    ) -> Result<bool, Self::Error>;
}

/// [`Process`] matching the [`Opt`](crate::set::Ctor::Opt) with [`Match`], and return the first matched
/// [`Match`] if successful.
pub trait Process<M: Match> {
    type Set: Set;
    type Error: Into<Error>;

    fn reset(&mut self);

    fn quit(&self) -> bool;

    fn count(&self) -> usize;

    fn sty(&self) -> Style;

    fn is_mat(&self) -> bool;

    fn consume(&self) -> bool;

    fn add_mat(&mut self, mat: M) -> &mut Self;

    fn mat(&self, index: usize) -> Option<&M>;

    fn mat_mut(&mut self, index: usize) -> Option<&mut M>;

    fn undo(&mut self, set: &mut Self::Set) -> Result<(), Self::Error>;

    fn process(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Option<usize>, Self::Error>;
}
