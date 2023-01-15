use std::fmt::Debug;

use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Infer;
use crate::set::Ctor;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetExt;
use crate::value::ValAccessor;
use crate::value::ValInitializer;
use crate::value::ValValidator;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Create option using given configurations.
pub struct Commit<'a, S, U>
where
    S: Set,
    U: Infer,
    SetCfg<S>: ConfigValue + Default,
{
    info: SetCfg<S>,
    set: &'a mut S,
    commited: Option<Uid>,
    pub(crate) drop_commit: bool,
    pub(crate) validator: Option<ValValidator<U::Val>>,
    pub(crate) initializer: Option<ValInitializer>,
}

impl<'a, S, U> Debug for Commit<'a, S, U>
where
    U: Infer,
    S: Set + Debug,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commit")
            .field("info", &self.info)
            .field("set", &self.set)
            .field("commited", &self.commited)
            .field("drop_commit", &self.drop_commit)
            .field("validator", &self.validator)
            .field("initializer", &self.initializer)
            .finish()
    }
}

impl<'a, S, U> Commit<'a, S, U>
where
    S: Set,
    U: Infer,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(set: &'a mut S, info: SetCfg<S>) -> Self {
        let initializer = U::infer_initializer();
        let validator = U::infer_validator();
        let info = Self::fill_infer_data(info);

        Self {
            set,
            info,
            commited: None,
            drop_commit: true,
            validator: validator,
            initializer: initializer,
        }
    }

    pub(crate) fn fill_infer_data(mut info: SetCfg<S>) -> SetCfg<S> {
        let act = U::infer_act();
        let styles = U::infer_style();
        let ignore_name = U::infer_ignore_name();
        let ignore_index = U::infer_ignore_index();
        let ignore_alias = U::infer_ignore_alias();

        info.set_type::<U::Val>();
        info.set_action(act);
        info.set_style(styles);
        info.set_ignore_name(ignore_name);
        info.set_ignore_index(ignore_index);
        info.set_ignore_alias(ignore_alias);
        info
    }

    pub fn cfg(&self) -> &SetCfg<S> {
        &self.info
    }

    pub fn cfg_mut(&mut self) -> &mut SetCfg<S> {
        &mut self.info
    }

    /// Set the option index of commit configuration.
    pub fn set_idx(mut self, index: Index) -> Self {
        self.info.set_idx(index);
        self
    }

    /// Set the option value action.
    pub fn set_action(mut self, action: Action) -> Self {
        self.info.set_action(action);
        self
    }

    /// Set the option name of commit configuration.
    pub fn set_name<T: Into<Str>>(mut self, name: T) -> Self {
        self.info.set_name(name);
        self
    }

    /// Set the option type name of commit configuration.
    pub fn set_type<T: ErasedTy>(mut self) -> Self {
        self.info.set_type::<T>();
        self
    }

    /// Clear all the alias of commit configuration.
    pub fn clr_alias(mut self) -> Self {
        self.info.clr_alias();
        self
    }

    /// Remove the given alias of commit configuration.
    pub fn rem_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.info.rem_alias(alias);
        self
    }

    /// Add given alias into the commit configuration.
    pub fn add_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.info.add_alias(alias);
        self
    }

    /// Set the option optional of commit configuration.
    pub fn set_force(mut self, force: bool) -> Self {
        self.info.set_force(force);
        self
    }

    /// Set the option hint message of commit configuration.
    pub fn set_hint<T: Into<Str>>(mut self, hint: T) -> Self {
        self.info.set_hint(hint);
        self
    }

    /// Set the option help message of commit configuration.
    pub fn set_help<T: Into<Str>>(mut self, help: T) -> Self {
        self.info.set_help(help);
        self
    }

    /// Set the option value initiator.
    pub fn set_initializer(mut self, initializer: ValInitializer) -> Self {
        self.initializer = Some(initializer);
        self
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        if let Some(commited) = self.commited {
            Ok(commited)
        } else {
            self.drop_commit = false;
            self.info.set_accessor(Some(ValAccessor::from_option(
                self.initializer.take(),
                self.validator.take(),
            )));
            let info = std::mem::take(&mut self.info);
            let opt = self
                .set
                .ctor_mut::<U::Val>()?
                .new_with(info)
                .map_err(|e| e.into())?;
            let uid = self.set.insert(opt);

            crate::trace_log!("Register a opt {:?} --> {}", info.name().cloned(), uid);
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

impl<'a, S, U> Commit<'a, S, U>
where
    S: Set,
    U: Infer,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(mut self, validator: ValValidator<U::Val>) -> Self {
        self.validator = Some(validator);
        self
    }
}

impl<'a, S, U> Commit<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: Copy,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}
impl<'a, S, U> Commit<'a, S, U>
where
    S: Set,
    U: Infer,
    U::Val: Clone,
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

impl<'a, S, U> Drop for Commit<'a, S, U>
where
    S: Set,
    U: Infer,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop_commit && self.commited.is_none() {
            let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}
