pub(crate) mod commit;
pub(crate) mod filter;
pub(crate) mod index;
pub(crate) mod optset;
pub(crate) mod optvalid;

pub use self::commit::Commit;
pub use self::filter::Filter;
pub use self::filter::FilterMatcher;
pub use self::filter::FilterMut;
pub use self::index::SetIndex;
pub use self::optset::OptSet;
pub use self::optvalid::OptValidator;
pub use self::optvalid::PrefixOptValidator;

use std::fmt::Debug;
use std::slice::Iter;
use std::slice::IterMut;

use crate::opt::Opt;
use crate::Error;
use crate::Str;
use crate::Uid;

/// An type alias for `<<I as Set>::Ctor as Ctor>::Opt`
pub type SetOpt<I> = <<I as Set>::Ctor as Ctor>::Opt;
/// An type alias for `<<I as Set>::Ctor as Ctor>::Config`
pub type SetCfg<I> = <<I as Set>::Ctor as Ctor>::Config;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        /// Implement [`Ctor`] for `Box<dyn Ctor>`.
        impl<Opt: crate::opt::Opt, Config: Send + Sync, Err: Into<Error>> Ctor
        for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err> + Send + Sync>
        {
            type Opt = Opt;

            type Config = Config;

            type Error = Err;

            fn r#type(&self) -> Str {
                Ctor::r#type(self.as_ref())
            }

            fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
                Ctor::new_with(self.as_mut(), config)
            }
        }

        impl<Opt: crate::opt::Opt, Config: Send + Sync, Err: Into<Error>> Debug
        for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err> + Send + Sync>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("Ctor")
                    .field(&format!("{{{}}}", self.r#type()))
                    .finish()
            }
        }
    }
    else {
        /// Implement [`Ctor`] for `Box<dyn Ctor>`.
        impl<Opt: crate::opt::Opt, Config, Err: Into<Error>> Ctor
        for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err>>
        {
            type Opt = Opt;

            type Config = Config;

            type Error = Err;

            fn r#type(&self) -> Str {
                Ctor::r#type(self.as_ref())
            }

            fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
                Ctor::new_with(self.as_mut(), config)
            }
        }

        impl<Opt: crate::opt::Opt, Config, Err: Into<Error>> Debug
        for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err>>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("Ctor")
                    .field(&format!("{{{}}}", self.r#type()))
                    .finish()
            }
        }
    }
}

/// Create [`Opt`](crate::set::Ctor::Opt) with given [`Config`](crate::set::Ctor::Config).
pub trait Ctor {
    type Opt: Opt;
    type Config;
    type Error: Into<Error>;

    fn r#type(&self) -> Str;

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error>;
}

impl<T: Ctor> From<T> for Str {
    fn from(c: T) -> Self {
        c.r#type()
    }
}

/// A collection store the [`Ctor`](Set::Ctor) and [`Opt`](Ctor::Opt).
pub trait Set {
    type Ctor: Ctor;

    /// Register a option creator type into option set.
    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor>;

    fn ctor_iter(&self) -> Iter<'_, Self::Ctor>;

    fn ctor_iter_mut(&mut self) -> IterMut<'_, Self::Ctor>;

    fn contain_ctor<S: Into<Str>>(&self, type_name: S) -> bool {
        let type_name = type_name.into();
        self.ctor_iter().any(|v| v.r#type() == type_name)
    }

    fn get_ctor<S: Into<Str>>(&self, type_name: S) -> Option<&Self::Ctor> {
        let type_name = type_name.into();

        self.ctor_iter().find(|v| v.r#type() == type_name)
    }

    fn get_ctor_mut<S: Into<Str>>(&mut self, type_name: S) -> Option<&mut Self::Ctor> {
        let type_name = type_name.into();

        self.ctor_iter_mut().find(|v| v.r#type() == type_name)
    }

    fn reset(&mut self);

    /// Return the number of options.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return all the unique id of option set.
    fn keys(&self) -> Vec<Uid> {
        self.iter().map(|v| v.uid()).collect()
    }

    fn iter(&self) -> Iter<'_, SetOpt<Self>>;

    fn iter_mut(&mut self) -> IterMut<'_, SetOpt<Self>>;

    fn contain(&self, uid: Uid) -> bool {
        self.iter().any(|v| v.uid() == uid)
    }

    fn insert(&mut self, opt: SetOpt<Self>) -> Uid;

    fn get(&self, id: Uid) -> Option<&SetOpt<Self>> {
        self.iter().find(|v| v.uid() == id)
    }

    fn get_mut(&mut self, id: Uid) -> Option<&mut SetOpt<Self>> {
        self.iter_mut().find(|v| v.uid() == id)
    }
}

pub trait SetExt<C: Ctor> {
    fn opt(&self, id: Uid) -> Result<&C::Opt, Error>;

    fn opt_mut(&mut self, id: Uid) -> Result<&mut C::Opt, Error>;

    fn ctor<S: Into<Str>>(&self, type_name: S) -> Result<&C, Error>;

    fn ctor_mut<S: Into<Str>>(&mut self, type_name: S) -> Result<&mut C, Error>;
}

impl<S: Set> SetExt<S::Ctor> for S {
    fn opt(&self, id: Uid) -> Result<&<S::Ctor as Ctor>::Opt, Error> {
        self.get(id)
            .ok_or_else(|| Error::raise_error(format!("Invalid uid {id} for Set")))
    }

    fn opt_mut(&mut self, id: Uid) -> Result<&mut <S::Ctor as Ctor>::Opt, Error> {
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
