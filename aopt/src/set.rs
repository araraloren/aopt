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

use crate::opt::Creator;
use crate::Error;
use crate::Str;
use crate::Uid;

/// A collection store the [`Ctor`](Set::Ctor) and [`Opt`](Creator::Opt).
pub trait Set {
    type Ctor: Creator;

    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor>;

    fn get_ctors(&self) -> &[Self::Ctor];

    fn get_ctors_mut(&mut self) -> &mut [Self::Ctor];

    fn contain_ctor<S: Into<Str>>(&self, type_name: S) -> bool {
        let type_name = type_name.into();
        self.get_ctors().iter().any(|v| v.r#type() == type_name)
    }

    fn get_ctor<S: Into<Str>>(&self, type_name: S) -> Option<&Self::Ctor> {
        let type_name = type_name.into();

        self.get_ctors().iter().find(|v| v.r#type() == type_name)
    }

    fn get_ctor_mut<S: Into<Str>>(&mut self, type_name: S) -> Option<&mut Self::Ctor> {
        let type_name = type_name.into();

        self.get_ctors_mut()
            .iter_mut()
            .find(|v| v.r#type() == type_name)
    }

    fn reset(&mut self);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn keys(&self) -> &[Uid];

    fn contain(&self, uid: Uid) -> bool {
        self.keys().iter().any(|v| v == &uid)
    }

    fn insert(&mut self, opt: <Self::Ctor as Creator>::Opt) -> Uid;

    fn get(&self, id: Uid) -> Option<&<Self::Ctor as Creator>::Opt>;

    fn get_mut(&mut self, id: Uid) -> Option<&mut <Self::Ctor as Creator>::Opt>;
}

pub trait SetExt<Ctor: Creator> {
    fn opt(&self, id: Uid) -> Result<&Ctor::Opt, Error>;

    fn opt_mut(&mut self, id: Uid) -> Result<&mut Ctor::Opt, Error>;

    fn ctor<S: Into<Str>>(&self, type_name: S) -> Result<&Ctor, Error>;

    fn ctor_mut<S: Into<Str>>(&mut self, type_name: S) -> Result<&mut Ctor, Error>;
}

impl<S: Set> SetExt<S::Ctor> for S {
    fn opt(&self, id: Uid) -> Result<&<S::Ctor as Creator>::Opt, Error> {
        self.get(id)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {id} for Set")))
    }

    fn opt_mut(&mut self, id: Uid) -> Result<&mut <S::Ctor as Creator>::Opt, Error> {
        self.get_mut(id)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {id} for Set")))
    }

    fn ctor<T: Into<Str>>(&self, type_name: T) -> Result<&S::Ctor, Error> {
        let type_name: Str = type_name.into();
        self.get_ctor(type_name.clone())
            .ok_or_else(|| Error::con_unsupport_option_type(type_name))
    }

    fn ctor_mut<T: Into<Str>>(&mut self, type_name: T) -> Result<&mut S::Ctor, Error> {
        let type_name: Str = type_name.into();
        self.get_ctor_mut(type_name.clone())
            .ok_or_else(|| Error::con_unsupport_option_type(type_name))
    }
}

pub trait Pre {
    fn prefix(&self) -> &[Str];

    fn add_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self;
}
