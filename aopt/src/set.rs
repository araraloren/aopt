pub(crate) mod commit;
pub(crate) mod filter;
pub(crate) mod index;
pub(crate) mod optset;
pub(crate) mod optvalid;

pub use self::commit::SetCommit;
pub use self::commit::SetCommitWithValue;
pub use self::filter::Filter;
pub use self::filter::FilterMatcher;
pub use self::filter::FilterMut;
pub use self::index::SetIndex;
pub use self::optset::OptSet;
pub use self::optvalid::OptValidator;
pub use self::optvalid::PrefixOptValidator;

use std::any::type_name;
use std::fmt::Debug;
use std::slice::Iter;
use std::slice::IterMut;

use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Opt;
use crate::opt::OptValueExt;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::Error;
use crate::Str;
use crate::Uid;

/// An type alias for `<<I as Set>::Ctor as Ctor>::Opt`
pub type SetOpt<I> = <<I as Set>::Ctor as Ctor>::Opt;
/// An type alias for `<<I as Set>::Ctor as Ctor>::Config`
pub type SetCfg<I> = <<I as Set>::Ctor as Ctor>::Config;

#[cfg(feature = "sync")]
/// Implement [`Ctor`] for `Box<dyn Ctor>`.
impl<Opt: crate::opt::Opt, Config: Send + Sync, Err: Into<Error>> Ctor
    for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err> + Send + Sync>
{
    type Opt = Opt;

    type Config = Config;

    type Error = Err;

    fn name(&self) -> &Str {
        Ctor::name(self.as_ref())
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        Ctor::new_with(self.as_mut(), config)
    }
}

#[cfg(feature = "sync")]
impl<Opt: crate::opt::Opt, Config: Send + Sync, Err: Into<Error>> Debug
    for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err> + Send + Sync>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ctor").finish()
    }
}

#[cfg(not(feature = "sync"))]
/// Implement [`Ctor`] for `Box<dyn Ctor>`.
impl<Opt: crate::opt::Opt, Config, Err: Into<Error>> Ctor
    for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err>>
{
    type Opt = Opt;

    type Config = Config;

    type Error = Err;

    fn name(&self) -> &Str {
        Ctor::name(self.as_ref())
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        Ctor::new_with(self.as_mut(), config)
    }
}

#[cfg(not(feature = "sync"))]
impl<Opt: crate::opt::Opt, Config, Err: Into<Error>> Debug
    for Box<dyn Ctor<Opt = Opt, Config = Config, Error = Err>>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ctor").finish()
    }
}

pub const CTOR_DEFAULT: &str = "default_ctor";

/// Get default creator name.
pub fn ctor_default_name() -> Str {
    Str::from(CTOR_DEFAULT)
}

/// Create [`Opt`](crate::set::Ctor::Opt) with given [`Config`](crate::set::Ctor::Config).
pub trait Ctor {
    type Opt: Opt;
    type Config;
    type Error: Into<Error>;

    fn name(&self) -> &Str;

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error>;
}

/// A collection of [`Ctor`](Set::Ctor) and [`Opt`](Ctor::Opt).
pub trait Set {
    type Ctor: Ctor;

    /// Register a option creator type into option set.
    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor>;

    fn ctor_iter(&self) -> Iter<'_, Self::Ctor>;

    fn ctor_iter_mut(&mut self) -> IterMut<'_, Self::Ctor>;

    fn contain_ctor(&self, name: &Str) -> bool {
        self.ctor_iter().any(|v| v.name() == name)
    }

    fn get_ctor(&self, name: &Str) -> Option<&Self::Ctor> {
        self.ctor_iter().find(|v| v.name() == name)
    }

    fn get_ctor_mut(&mut self, name: &Str) -> Option<&mut Self::Ctor> {
        self.ctor_iter_mut().find(|v| v.name() == name)
    }

    fn reset(&mut self);

    /// Return the number of options.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return all the unique uid of option set.
    fn keys(&self) -> Vec<Uid> {
        self.iter().map(|v| v.uid()).collect()
    }

    fn iter(&self) -> Iter<'_, SetOpt<Self>>;

    fn iter_mut(&mut self) -> IterMut<'_, SetOpt<Self>>;

    fn contain(&self, uid: Uid) -> bool {
        self.iter().any(|v| v.uid() == uid)
    }

    fn insert(&mut self, opt: SetOpt<Self>) -> Uid;

    fn get(&self, uid: Uid) -> Option<&SetOpt<Self>> {
        self.iter().find(|v| v.uid() == uid)
    }

    fn get_mut(&mut self, uid: Uid) -> Option<&mut SetOpt<Self>> {
        self.iter_mut().find(|v| v.uid() == uid)
    }
}

pub trait SetExt<C: Ctor> {
    fn opt(&self, uid: Uid) -> Result<&C::Opt, Error>;

    fn opt_mut(&mut self, uid: Uid) -> Result<&mut C::Opt, Error>;

    fn ctor(&self, name: &Str) -> Result<&C, Error>;

    fn ctor_mut(&mut self, name: &Str) -> Result<&mut C, Error>;
}

impl<S: Set> SetExt<S::Ctor> for S {
    fn opt(&self, uid: Uid) -> Result<&<S::Ctor as Ctor>::Opt, Error> {
        self.get(uid)
            .ok_or_else(|| Error::raise_error(format!("Can not find option `{}` by uid", uid)))
    }

    fn opt_mut(&mut self, uid: Uid) -> Result<&mut <S::Ctor as Ctor>::Opt, Error> {
        self.get_mut(uid)
            .ok_or_else(|| Error::raise_error(format!("Can not find option `{}` by uid", uid)))
    }

    fn ctor(&self, name: &Str) -> Result<&S::Ctor, Error> {
        self.get_ctor(name)
            .ok_or_else(|| Error::raise_error(format!("Can not find option `{}` by name", name)))
    }

    fn ctor_mut(&mut self, name: &Str) -> Result<&mut S::Ctor, Error> {
        self.get_ctor_mut(name)
            .ok_or_else(|| Error::raise_error(format!("Can not find option `{}` by name", name)))
    }
}

pub trait SetValueFindExt
where
    Self: Set + Sized,
{
    fn find_uid<S: Into<Str>>(&self, opt: S) -> Result<Uid, Error>;

    fn find_val<U: ErasedTy>(&self, opt: impl Into<Str>) -> Result<&U, Error> {
        self.opt(self.find_uid(opt)?)?.val::<U>()
    }

    fn find_val_mut<U: ErasedTy>(&mut self, opt: impl Into<Str>) -> Result<&mut U, Error> {
        self.opt_mut(self.find_uid(opt)?)?.val_mut()
    }

    fn find_vals<U: ErasedTy>(&self, opt: impl Into<Str>) -> Result<&Vec<U>, Error> {
        self.opt(self.find_uid(opt)?)?.vals()
    }

    fn find_vals_mut<U: ErasedTy>(&mut self, opt: impl Into<Str>) -> Result<&mut Vec<U>, Error> {
        self.opt_mut(self.find_uid(opt)?)?.vals_mut()
    }

    fn take_val<U: ErasedTy>(&mut self, opt: impl Into<Str>) -> Result<U, Error> {
        let name: Str = opt.into();
        let opt = self.find_uid(name.clone())?;
        let vals = self.opt_mut(opt)?.vals_mut::<U>()?;

        vals.pop().ok_or_else(|| {
            Error::raise_error(format!(
                "Not enough value({}) can take from option `{}`",
                type_name::<U>(),
                name
            ))
        })
    }
}

pub trait Commit<S: Set>
where
    Self: Sized,
    SetCfg<S>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<S>;

    fn cfg_mut(&mut self) -> &mut SetCfg<S>;

    fn set_index(mut self, index: Index) -> Self {
        self.cfg_mut().set_index(index);
        self
    }

    fn set_action(mut self, action: Action) -> Self {
        self.cfg_mut().set_action(action);
        self
    }

    fn set_name<T: Into<Str>>(mut self, name: T) -> Self {
        self.cfg_mut().set_name(name);
        self
    }

    fn set_ctor<T: Into<Str>>(mut self, ctor: T) -> Self {
        self.cfg_mut().set_ctor(ctor);
        self
    }

    fn clr_alias(mut self) -> Self {
        self.cfg_mut().clr_alias();
        self
    }

    fn rem_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.cfg_mut().rem_alias(alias);
        self
    }

    fn add_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.cfg_mut().add_alias(alias);
        self
    }

    fn set_force(mut self, force: bool) -> Self {
        self.cfg_mut().set_force(force);
        self
    }

    fn set_hint<T: Into<Str>>(mut self, hint: T) -> Self {
        self.cfg_mut().set_hint(hint);
        self
    }

    fn set_help<T: Into<Str>>(mut self, help: T) -> Self {
        self.cfg_mut().set_help(help);
        self
    }

    fn set_storer(mut self, storer: ValStorer) -> Self {
        self.cfg_mut().set_storer(storer);
        self
    }

    fn set_initializer<T: Into<ValInitializer>>(mut self, initializer: T) -> Self {
        self.cfg_mut().set_initializer(initializer.into());
        self
    }
}

pub trait SetChecker<S> {
    type Error: Into<Error>;

    fn pre_check(&self, set: &mut S) -> Result<bool, Self::Error>;

    fn opt_check(&self, set: &mut S) -> Result<bool, Self::Error>;

    fn pos_check(&self, set: &mut S) -> Result<bool, Self::Error>;

    fn cmd_check(&self, set: &mut S) -> Result<bool, Self::Error>;

    fn post_check(&self, set: &mut S) -> Result<bool, Self::Error>;
}
