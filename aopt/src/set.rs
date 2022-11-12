pub(crate) mod commit;
pub(crate) mod filter;
pub(crate) mod index;
pub(crate) mod optset;

pub use self::commit::Commit;
pub use self::filter::Filter;
pub use self::filter::FilterMatcher;
pub use self::filter::FilterMut;
pub use self::index::SetIndex;
pub use self::optset::OptSet;

use crate::Error;
use crate::Str;
use crate::Uid;

/// A collection store the [`Set::Opt`].
pub trait Set {
    type Opt;

    fn reset(&mut self);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn keys(&self) -> &[Uid];

    fn contain(&self, uid: Uid) -> bool {
        self.keys().iter().any(|v| v == &uid)
    }

    fn insert(&mut self, opt: Self::Opt) -> Uid;

    fn get(&self, id: Uid) -> Option<&Self::Opt>;

    fn get_mut(&mut self, id: Uid) -> Option<&mut Self::Opt>;
}

pub trait SetExt<Opt> {
    fn opt(&self, id: Uid) -> Result<&Opt, Error>;

    fn opt_mut(&mut self, id: Uid) -> Result<&mut Opt, Error>;
}

impl<S: Set> SetExt<S::Opt> for S {
    fn opt(&self, id: Uid) -> Result<&S::Opt, Error> {
        self.get(id)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {id} for Set")))
    }

    fn opt_mut(&mut self, id: Uid) -> Result<&mut S::Opt, Error> {
        self.get_mut(id)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {id} for Set")))
    }
}

pub trait Pre {
    fn prefix(&self) -> &[Str];

    fn add_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self;
}
