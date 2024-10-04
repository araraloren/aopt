use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::opt::Any;
use crate::opt::Cmd;
use crate::opt::ConfigValue;
use crate::opt::Main;
use crate::opt::Pos;
use crate::prelude::ErasedTy;
use crate::set::Ctor;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetExt;
use crate::trace;
use crate::value::Infer;
use crate::value::Placeholder;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::value::ValValidator;
use crate::Error;
use crate::Uid;

use super::Commit;

/// Create option using given configurations.
pub struct SetCommit<'a, S, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    info: Option<SetCfg<S>>,
    set: Option<&'a mut S>,
    uid: Option<Uid>,
    pub(crate) drop: bool,
    marker: PhantomData<U>,
}

impl<'a, S, U> Debug for SetCommit<'a, S, U>
where
    S: Set + Debug,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetCommitW")
            .field("info", &self.info)
            .field("set", &self.set)
            .field("uid", &self.uid)
            .field("drop", &self.drop)
            .finish()
    }
}

macro_rules! add_interface {
    ($ty:ty, $name1:ident, $name2:ident) => {
        #[doc = concat!("Set the infer type to [`", stringify!($ty), "`]\\<T\\>.")]
        pub fn $name1<T>(
            self,
        ) -> SetCommit<'a, S, $ty> where T::Val: RawValParser, T: ErasedTy + Infer {
            let type_id = self.cfg().r#type();

            debug_assert!(
                type_id.is_none() || type_id == Some(&TypeId::of::<$ty>()),
                "Can not set value type of {} if it already has one", stringify!($ty),
            );
            self.set_infer::<$ty>()
        }

        #[doc = concat!("Set the infer type to [`", stringify!($ty) ,"`]\\<T\\>, add default initializer and default storer.")]
        ///
        /// The function will call [`add_default_initializer`](SetCommit::add_default_initializer) add
        /// [`add_default_storer`](SetCommit::add_default_storer).
        pub fn $name2<T>(
            self,
        ) -> SetCommit<'a, S, $ty> where T::Val: RawValParser + Clone, T: ErasedTy + Infer {
            let type_id = self.cfg().r#type();

            debug_assert!(
                type_id.is_none() || type_id == Some(&TypeId::of::<$ty>()),
                "Can not set value type of {} if it already has one", stringify!($ty),
            );
            self.set_infer::<$ty>()
                .add_default_initializer()
                .add_default_storer()
        }
    }
}

impl<'a, S> SetCommit<'a, S, Placeholder>
where
    S: Set,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new_placeholder(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self {
            set: Some(set),
            info: Some(info),
            uid: None,
            drop: true,
            marker: PhantomData,
        }
    }

    add_interface!(Pos<T>, set_pos_type_only, set_pos_type);

    add_interface!(Main<T>, set_main_type_only, set_main_type);

    add_interface!(Any<T>, set_any_type_only, set_any_type);
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self {
            set: Some(set),
            info: Some(info),
            uid: None,
            drop: true,
            marker: PhantomData,
        }
    }

    /// Set the infer type of option.
    pub fn set_infer<O>(mut self) -> SetCommit<'a, S, O>
    where
        O: Infer + 'static,
        O::Val: RawValParser,
    {
        self.drop = false;

        let set = self.set.take();
        let info = self.info.take();
        let info = info.unwrap();

        SetCommit::new(set.unwrap(), info)
    }

    pub(crate) fn commit_change(&mut self) -> Result<Uid, Error> {
        if let Some(uid) = self.uid {
            Ok(uid)
        } else {
            self.drop = false;

            let info = std::mem::take(&mut self.info);
            let mut info = info.unwrap();

            <U as Infer>::infer_fill_info(&mut info)?;

            let set = self.set.as_mut().unwrap();
            let ctor = info.ctor().ok_or_else(|| {
                crate::raise_error!("Invalid configuration: missing creator name!")
            })?;

            trace!("Register a opt {:?} with creator({})", info.name(), ctor);

            let opt = set.ctor_mut(ctor)?.new_with(info).map_err(|e| e.into())?;
            let uid = set.insert(opt);

            trace!("--> register okay: {uid}");
            self.uid = Some(uid);
            Ok(uid)
        }
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`].
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.commit_change()
    }
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the value type of option(except for [`Cmd`]).
    pub fn set_value_type_only<T: ErasedTy>(self) -> SetCommitWithValue<'a, S, U, T> {
        debug_assert!(
            TypeId::of::<U>() != TypeId::of::<Cmd>() || TypeId::of::<T>() == TypeId::of::<bool>(),
            "For Cmd, you can't have other value type!"
        );
        SetCommitWithValue::new(self)
    }

    /// Set the value type of option, add default initializer and default storer.
    ///
    /// The function will call [`add_default_initializer_t`](SetCommitWithValue::add_default_initializer_t) add
    /// [`add_default_storer_t`](SetCommitWithValue::add_default_storer_t).
    pub fn set_value_type<T: ErasedTy + RawValParser + Clone>(
        self,
    ) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type_only::<T>()
            .add_default_initializer_t()
            .add_default_storer_t()
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type_only::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Clone>(self, value: T) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type_only::<T>().set_value_t(value)
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(
        self,
        value: Vec<T>,
    ) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type_only::<T>().set_values_t(value)
    }
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }

    /// Add default [`storer`](ValStorer::fallback) of type [`U::Val`](Infer::Val).
    pub fn add_default_storer(self) -> Self {
        self.set_storer(ValStorer::fallback::<U::Val>())
    }
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: Clone + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::new_value(value))
    }

    /// Set the option default value.
    pub fn set_values(self, value: Vec<U::Val>) -> Self {
        self.set_initializer(ValInitializer::new_values(value))
    }

    /// Add a default [`initializer`](ValInitializer::fallback).
    pub fn add_default_initializer(self) -> Self {
        self.set_initializer(ValInitializer::fallback())
    }
}

impl<'a, S, U> Commit<S> for SetCommit<'a, S, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<S> {
        self.info.as_ref().unwrap()
    }

    fn cfg_mut(&mut self) -> &mut SetCfg<S> {
        self.info.as_mut().unwrap()
    }
}

impl<'a, S, U> Drop for SetCommit<'a, S, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop {
            self.commit_change()
                .unwrap_or_else(|e| panic!("catch error in SetCommit::drop: {:?}", e));
        }
    }
}

/// Create option using given configurations.
pub struct SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    inner: Option<SetCommit<'a, S, U>>,

    marker: PhantomData<T>,
}

impl<'a, S, U, T> Debug for SetCommitWithValue<'a, S, U, T>
where
    U: Infer + 'static,
    T: ErasedTy,
    S: Set + Debug,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetCommitWithValue")
            .field("inner", &self.inner)
            .field("marker", &self.marker)
            .finish()
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(inner: SetCommit<'a, S, U>) -> Self {
        Self {
            inner: Some(inner),
            marker: PhantomData,
        }
    }

    pub fn inner(&self) -> Result<&SetCommit<'a, S, U>, Error> {
        self.inner
            .as_ref()
            .ok_or_else(|| crate::raise_error!("Must set inner data of SetCommitWithValue(ref)"))
    }

    pub fn inner_mut(&mut self) -> Result<&mut SetCommit<'a, S, U>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| crate::raise_error!("Must set inner data of SetCommitWithValue(mut)"))
    }

    /// Set the infer type of option.
    pub fn set_infer<O: Infer>(mut self) -> SetCommitWithValue<'a, S, O, T>
    where
        O::Val: RawValParser,
    {
        SetCommitWithValue::new(self.inner.take().unwrap().set_infer::<O>())
    }

    pub(crate) fn commit_inner_change(&mut self) -> Result<Uid, Error> {
        self.inner_mut()?.commit_change()
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`].
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.commit_inner_change()
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    T: ErasedTy + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator_t(self, validator: ValValidator<T>) -> Self {
        self.set_storer(ValStorer::new_validator(validator))
    }

    /// Add default [`storer`](ValStorer::fallback) of type `T`.
    pub fn add_default_storer_t(self) -> Self {
        self.set_storer(ValStorer::fallback::<T>())
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy + Clone,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::new_value(value))
    }

    /// Set the option default value.
    pub fn set_values_t(self, value: Vec<T>) -> Self {
        self.set_initializer(ValInitializer::new_values(value))
    }

    /// Add a default [`initializer`](ValInitializer::fallback).
    pub fn add_default_initializer_t(self) -> Self {
        self.set_initializer(ValInitializer::fallback())
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer + 'static,
    U::Val: Clone + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::new_value(value))
    }

    /// Set the option default value.
    pub fn set_values(self, value: Vec<U::Val>) -> Self {
        self.set_initializer(ValInitializer::new_values(value))
    }
}

impl<'a, S, U, T> Commit<S> for SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer + 'static,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<S> {
        self.inner().unwrap().cfg()
    }

    fn cfg_mut(&mut self) -> &mut SetCfg<S> {
        self.inner_mut().unwrap().cfg_mut()
    }
}
