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
pub struct SetCommitW<'a, S, U>
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
    pub(crate) infer: bool,
    marker: PhantomData<U>,
}

impl<'a, S, U> Debug for SetCommitW<'a, S, U>
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
            .field("infer", &self.infer)
            .field("marker", &self.marker)
            .finish()
    }
}

impl<'a, S> SetCommitW<'a, S, Placeholder>
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
            infer: true,
            marker: PhantomData::default(),
        }
    }
}

impl<'a, S, U> SetCommitW<'a, S, U>
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
            infer: false,
            marker: PhantomData::default(),
        }
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> SetCommitW<'a, S, O>
    where
        O::Val: RawValParser,
    {
        self.drop = false;

        let set = self.set.take();
        let info = self.info.take();

        SetCommitW::new(set.unwrap(), info.unwrap())
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        if let Some(commited) = self.uid {
            Ok(commited)
        } else {
            self.drop = false;

            let info = std::mem::take(&mut self.info);
            let mut info = info.unwrap();
            let set = self.set.as_mut().unwrap();

            info.set_fix_infer(self.infer);

            let _name = info.name().cloned();
            let ctor = info.ctor().ok_or_else(|| {
                Error::raise_error("Invalid configuration: missing creator name!")
            })?;
            let opt = set.ctor_mut(ctor)?.new_with(info).map_err(|e| e.into())?;
            let uid = set.insert(opt);

            crate::trace_log!("Register a opt {:?} --> {}", _name, uid);
            self.uid = Some(uid);
            Ok(uid)
        }
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`].
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.drop = false;
        self.run_and_commit_the_change()
    }
}

impl<'a, S, U> SetCommitW<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the type of option.
    pub fn set_value_type<T: ErasedTy>(mut self) -> SetCommitWT<'a, S, U, T> {
        self.drop = false;

        let set = self.set.take();
        let info = self.info.take();

        SetCommitWT::new(set.unwrap(), info.unwrap())
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> SetCommitWT<'a, S, U, T> {
        self.set_value_type::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Copy>(self, value: T) -> SetCommitWT<'a, S, U, T> {
        self.set_value_type::<T>().set_value_t(value)
    }

    /// Set the option default value.
    pub fn set_value_clone_t<T: ErasedTy + Clone>(self, value: T) -> SetCommitWT<'a, S, U, T> {
        self.set_value_type::<T>().set_value_clone_t(value)
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(self, value: Vec<T>) -> SetCommitWT<'a, S, U, T> {
        self.set_value_type::<T>().set_values_t(value)
    }
}

impl<'a, S, U> SetCommitW<'a, S, U>
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

impl<'a, S, U> SetCommitW<'a, S, U>
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

impl<'a, S, U> SetCommitW<'a, S, U>
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

impl<'a, S, U> Commit<S> for SetCommitW<'a, S, U>
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

impl<'a, S, U> Drop for SetCommitW<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop && self.uid.is_none() {
            let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}

/// Create option using given configurations.
pub struct SetCommitWT<'a, S, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    info: Option<SetCfg<S>>,
    set: Option<&'a mut S>,
    uid: Option<Uid>,
    pub(crate) drop: bool,
    pub(crate) infer: bool,
    marker: PhantomData<(U, T)>,
}

impl<'a, S, T> SetCommitWT<'a, S, Placeholder, T>
where
    S: Set,
    T: ErasedTy,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new_placeholder(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self {
            set: Some(set),
            info: Some(info),
            uid: None,
            drop: true,
            infer: true,
            marker: PhantomData::default(),
        }
    }
}

impl<'a, S, U, T> SetCommitWT<'a, S, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self {
            set: Some(set),
            info: Some(info),
            uid: None,
            drop: true,
            infer: false,
            marker: PhantomData::default(),
        }
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> SetCommitWT<'a, S, O, T>
    where
        O::Val: RawValParser,
    {
        self.drop = false;

        let set = self.set.take();
        let info = self.info.take();

        SetCommitWT::new(set.unwrap(), info.unwrap())
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        if let Some(commited) = self.uid {
            Ok(commited)
        } else {
            self.drop = false;

            let info = std::mem::take(&mut self.info);
            let mut info = info.unwrap();
            let set = self.set.as_mut().unwrap();

            info.set_fix_infer(self.infer);

            let _name = info.name().cloned();
            let ctor = info.ctor().ok_or_else(|| {
                Error::raise_error("Invalid configuration: missing creator name!")
            })?;
            let opt = set.ctor_mut(ctor)?.new_with(info).map_err(|e| e.into())?;
            let uid = set.insert(opt);

            crate::trace_log!("Register a opt {:?} --> {}", _name, uid);
            self.uid = Some(uid);
            Ok(uid)
        }
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`].
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.drop = false;
        self.run_and_commit_the_change()
    }
}

impl<'a, S, U, T> SetCommitWT<'a, S, U, T>
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

impl<'a, S, U, T> SetCommitWT<'a, S, U, T>
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
impl<'a, S, U, T> SetCommitWT<'a, S, U, T>
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

impl<'a, S, U, T> SetCommitWT<'a, S, U, T>
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

impl<'a, S, U, T> SetCommitWT<'a, S, U, T>
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

impl<'a, S, U, T> SetCommitWT<'a, S, U, T>
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

impl<'a, S, U, T> Commit<S> for SetCommitWT<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
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

impl<'a, S, U, T> Drop for SetCommitWT<'a, S, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop && self.uid.is_none() {
            let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}
