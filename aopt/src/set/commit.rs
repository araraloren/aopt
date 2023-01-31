use std::fmt::Debug;
use std::marker::PhantomData;

use crate::opt::config::fill_cfg;
use crate::opt::ConfigValue;
use crate::prelude::ErasedTy;
use crate::set::Ctor;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetExt;
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
    U: Infer,
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
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetCommitW")
            .field("info", &self.info)
            .field("set", &self.set)
            .field("uid", &self.uid)
            .field("drop", &self.drop)
            .field("marker", &self.marker)
            .finish()
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
            marker: PhantomData::default(),
        }
    }
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(set: &'a mut S, mut info: SetCfg<S>) -> Self {
        fill_cfg::<U, SetCfg<S>>(&mut info);
        Self {
            set: Some(set),
            info: Some(info),
            uid: None,
            drop: true,
            marker: PhantomData::default(),
        }
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> SetCommit<'a, S, O>
    where
        O::Val: RawValParser,
    {
        self.drop = false;

        let set = self.set.take();
        let info = self.info.take();

        SetCommit::new(set.unwrap(), info.unwrap())
    }

    pub(crate) fn commit_change(&mut self) -> Result<Uid, Error> {
        if let Some(uid) = self.uid {
            Ok(uid)
        } else {
            self.drop = false;

            let info = std::mem::take(&mut self.info);
            let info = info.unwrap();
            let set = self.set.as_mut().unwrap();
            let ctor = info
                .ctor()
                .ok_or_else(|| Error::raise_error("Invalid configuration: missing creator name!"))?
                .clone();

            crate::trace_log!("Register a opt {:?} with creator({})", info.name(), ctor);

            let opt = set.ctor_mut(&ctor)?.new_with(info).map_err(|e| e.into())?;
            let uid = set.insert(opt);

            crate::trace_log!("--> register okay: {uid}");
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
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the type of option.
    pub(crate) fn set_value_type<T: ErasedTy>(self) -> SetCommitWithValue<'a, S, U, T> {
        SetCommitWithValue::new(self)
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Copy>(self, value: T) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type::<T>().set_value_t(value)
    }

    /// Set the option default value.
    pub fn set_value_clone_t<T: ErasedTy + Clone>(
        self,
        value: T,
    ) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type::<T>().set_value_clone_t(value)
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(
        self,
        value: Vec<T>,
    ) -> SetCommitWithValue<'a, S, U, T> {
        self.set_value_type::<T>().set_values_t(value)
    }
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: Copy + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}

impl<'a, S, U> SetCommit<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: Clone + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_clone(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values(self, value: Vec<U::Val>) -> Self {
        self.set_initializer(ValInitializer::with_vec(value))
    }
}

impl<'a, S, U> Commit<S> for SetCommit<'a, S, U>
where
    S: Set,
    U: Infer,
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
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop {
            let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

            self.commit_change().expect(error);
        }
    }
}

/// Create option using given configurations.
pub struct SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    inner: Option<SetCommit<'a, S, U>>,

    marker: PhantomData<T>,
}

impl<'a, S, U, T> Debug for SetCommitWithValue<'a, S, U, T>
where
    U: Infer,
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
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(inner: SetCommit<'a, S, U>) -> Self {
        Self {
            inner: Some(inner),
            marker: PhantomData::default(),
        }
    }

    pub fn inner(&self) -> Result<&SetCommit<'a, S, U>, Error> {
        self.inner
            .as_ref()
            .ok_or_else(|| Error::raise_error("Must set inner data of SetCommitWithValue(ref)"))
    }

    pub fn inner_mut(&mut self) -> Result<&mut SetCommit<'a, S, U>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| Error::raise_error("Must set inner data of SetCommitWithValue(mut)"))
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> SetCommitWithValue<'a, S, O, T>
    where
        O::Val: RawValParser,
    {
        SetCommitWithValue::new(self.inner.take().unwrap().set_type::<O>())
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
    U: Infer,
    U::Val: RawValParser,
    T: ErasedTy + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator_t(mut self, validator: ValValidator<T>) -> Self {
        self.cfg_mut()
            .set_storer(ValStorer::new_validator(validator));
        self
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy + Copy,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}
impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy + Clone,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_clone_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values_t(self, value: Vec<T>) -> Self {
        self.set_initializer(ValInitializer::with_vec(value))
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer,
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
    U: Infer,
    U::Val: Copy + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}

impl<'a, S, U, T> SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer,
    U::Val: Clone + RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_clone(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values(self, value: Vec<U::Val>) -> Self {
        self.set_initializer(ValInitializer::with_vec(value))
    }
}

impl<'a, S, U, T> Commit<S> for SetCommitWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer,
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
