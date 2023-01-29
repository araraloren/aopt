use std::fmt::Debug;
use std::marker::PhantomData;

use crate::map::ErasedTy;
use crate::opt::config::fill_cfg;
use crate::opt::ConfigValue;
use crate::set::Ctor;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetExt;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::value::ValValidator;
use crate::Error;
use crate::Uid;

use super::Commit;

pub struct SetCommitInfered<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    info: Option<SetCfg<S>>,
    set: Option<&'a mut S>,
    commited: Option<Uid>,
    pub(crate) drop_commit: bool,
    marker: PhantomData<U>,
}
impl<'a, S, U> Debug for SetCommitInfered<'a, S, U>
where
    S: Set + Debug,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitInfered")
            .field("info", &self.info)
            .field("set", &self.set)
            .field("commited", &self.commited)
            .field("drop_commit", &self.drop_commit)
            .field("marker", &self.marker)
            .finish()
    }
}

impl<'a, S, U> SetCommitInfered<'a, S, U>
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
            commited: None,
            drop_commit: true,
            marker: PhantomData::default(),
        }
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        if let Some(commited) = self.commited {
            Ok(commited)
        } else {
            self.drop_commit = false;

            let info = std::mem::take(&mut self.info);
            let info = info.unwrap();
            let set = self.set.as_mut().unwrap();

            let _name = info.name().cloned();
            let ctor = info.ctor().ok_or_else(|| {
                Error::raise_error("Invalid configuration: missing creator name!")
            })?;
            let opt = set.ctor_mut(ctor)?.new_with(info).map_err(|e| e.into())?;
            let uid = set.insert(opt);

            crate::trace_log!("Register a opt {:?} --> {}", _name, uid);
            self.commited = Some(uid);
            Ok(uid)
        }
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`].
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.drop_commit = false;
        self.run_and_commit_the_change()
    }
}

impl<'a, S, U> Commit<S> for SetCommitInfered<'a, S, U>
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

impl<'a, S, U> SetCommitInfered<'a, S, U>
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

impl<'a, S, U> SetCommitInfered<'a, S, U>
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
impl<'a, S, U> SetCommitInfered<'a, S, U>
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

impl<'a, S, U> Drop for SetCommitInfered<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop_commit && self.commited.is_none() {
            let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}

/// Convert [`Commit`] to [`CommitWithValue`].
impl<'a, S, U> SetCommitInfered<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the type of option.
    fn set_value_type<T: ErasedTy>(mut self) -> SetCommitInferedWithValue<'a, S, U, T> {
        self.drop_commit = false;

        let set = self.set.take();
        let info = self.info.take();

        SetCommitInferedWithValue::new(set.unwrap(), info.unwrap())
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> SetCommitInferedWithValue<'a, S, U, T> {
        self.set_value_type::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Copy>(
        self,
        value: T,
    ) -> SetCommitInferedWithValue<'a, S, U, T> {
        self.set_value_type::<T>().set_value_t(value)
    }

    /// Set the option default value.
    pub fn set_value_clone_t<T: ErasedTy + Clone>(
        self,
        value: T,
    ) -> SetCommitInferedWithValue<'a, S, U, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(
        self,
        value: Vec<T>,
    ) -> SetCommitInferedWithValue<'a, S, U, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_vec(value))
    }
}

/// Create option using given configurations.
pub struct SetCommitInferedWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    info: Option<SetCfg<S>>,
    set: Option<&'a mut S>,
    commited: Option<Uid>,
    pub(crate) drop_commit: bool,
    marker: PhantomData<(U, T)>,
}

impl<'a, S, U, T> SetCommitInferedWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self {
            set: Some(set),
            info: Some(info),
            commited: None,
            drop_commit: true,
            marker: PhantomData::default(),
        }
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        if let Some(commited) = self.commited {
            Ok(commited)
        } else {
            self.drop_commit = false;

            let info = std::mem::take(&mut self.info);
            let info = info.unwrap();
            let set = self.set.as_mut().unwrap();

            let _name = info.name().cloned();
            let ctor = info.ctor().ok_or_else(|| {
                Error::raise_error("Invalid configuration: missing creator name!")
            })?;
            let opt = set.ctor_mut(ctor)?.new_with(info).map_err(|e| e.into())?;
            let uid = set.insert(opt);

            crate::trace_log!("Register a opt {:?} --> {}", _name, uid);
            self.commited = Some(uid);
            Ok(uid)
        }
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`].
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.drop_commit = false;
        self.run_and_commit_the_change()
    }
}

impl<'a, S, U, T> SetCommitInferedWithValue<'a, S, U, T>
where
    S: Set,
    T: ErasedTy + RawValParser,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator_t(mut self, validator: ValValidator<T>) -> Self {
        self.cfg_mut()
            .set_storer(ValStorer::new_validator(validator));
        self
    }
}

impl<'a, S, U, T> SetCommitInferedWithValue<'a, S, U, T>
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
impl<'a, S, U, T> SetCommitInferedWithValue<'a, S, U, T>
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

impl<'a, S, U, T> SetCommitInferedWithValue<'a, S, U, T>
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

impl<'a, S, U, T> SetCommitInferedWithValue<'a, S, U, T>
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

impl<'a, S, U, T> SetCommitInferedWithValue<'a, S, U, T>
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

impl<'a, S, U, T> Commit<S> for SetCommitInferedWithValue<'a, S, U, T>
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

impl<'a, S, U, T> Drop for SetCommitInferedWithValue<'a, S, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop_commit && self.commited.is_none() {
            let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}
