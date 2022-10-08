pub(crate) mod commit;
pub(crate) mod filter;
pub(crate) mod optset;

pub use self::commit::Commit;
pub use self::filter::Filter;
pub use self::filter::FilterMatcher;
pub use self::filter::FilterMut;
pub use self::optset::OptSet;

use crate::Error;
use crate::Str;
use crate::Uid;

/// A collection store the [`Set::Opt`].
pub trait Set {
    type Opt;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn has(&self, uid: Uid) -> bool {
        self.keys().iter().any(|v| v == &uid)
    }

    fn len(&self) -> usize;

    fn reset(&mut self);

    fn keys(&self) -> &[Uid];

    fn insert(&mut self, opt: Self::Opt) -> Uid;

    fn get(&self, id: Uid) -> Option<&Self::Opt>;

    fn get_mut(&mut self, id: Uid) -> Option<&mut Self::Opt>;
}

pub trait SetExt<Opt> {
    fn opt(&self, id: Uid) -> Result<&Opt, Error>;

    fn opt_mut(&mut self, id: Uid) -> Result<&mut Opt, Error>;
}

impl<Opt, S> SetExt<Opt> for S
where
    S: Set<Opt = Opt>,
{
    fn opt(&self, id: Uid) -> Result<&Opt, Error> {
        debug_assert!(self.has(id), "Invalid uid for Set");
        self.get(id)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {id} for Set")))
    }

    fn opt_mut(&mut self, id: Uid) -> Result<&mut Opt, Error> {
        debug_assert!(self.has(id), "Invalid uid for Set");
        self.get_mut(id)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {id} for Set")))
    }
}

/// Prefix using for parsing option string.
pub trait PreSet {
    fn pre(&self) -> &[Str];

    fn add_pre(&mut self, prefix: &str) -> &mut Self;
}
