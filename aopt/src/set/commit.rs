use std::fmt::Debug;

use crate::opt::Action;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::set::Ctor;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetExt;
use crate::set::UCommit;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::value::ValAccessor;
use crate::value::ValInitializer;
use crate::value::ValValidator;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Create option using given configurations.
pub struct Commit<'a, S>
where
    S: Set,
    SetCfg<S>: ConfigValue + Default,
{
    info: Option<SetCfg<S>>,
    set: Option<&'a mut S>,
    commited: Option<Uid>,
    pub(crate) drop_commit: bool,
    pub(crate) initializer: Option<ValInitializer>,
}

impl<'a, S> Debug for Commit<'a, S>
where
    S: Set + Debug,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commit")
            .field("info", &self.info)
            .field("set", &self.set)
            .field("commited", &self.commited)
            .field("drop_commit", &self.drop_commit)
            .field("initializer", &self.initializer)
            .finish()
    }
}

impl<'a, S> Commit<'a, S>
where
    S: Set,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self {
            set: Some(set),
            info: Some(info),
            commited: None,
            drop_commit: true,
            initializer: None,
        }
    }

    pub(crate) fn convert2infer<U: Infer>(mut self) -> UCommit<'a, S, U>
    where
        U::Val: RawValParser,
    {
        self.drop_commit = false;

        let set = self.set.take();
        let info = self.info.take();
        let initializer = self.initializer.take();

        let mut uc = UCommit::new(set.unwrap(), info.unwrap());

        uc.initializer = initializer;
        uc
    }

    pub fn cfg(&self) -> &SetCfg<S> {
        self.info.as_ref().unwrap()
    }

    pub fn cfg_mut(&mut self) -> &mut SetCfg<S> {
        self.info.as_mut().unwrap()
    }

    /// Set the option index of commit configuration.
    pub fn set_idx(mut self, index: Index) -> Self {
        self.cfg_mut().set_idx(index);
        self
    }

    /// Set the option value action.
    pub fn set_action(mut self, action: Action) -> Self {
        self.cfg_mut().set_action(action);
        self
    }

    /// Set the option name of commit configuration.
    pub fn set_name<T: Into<Str>>(mut self, name: T) -> Self {
        self.cfg_mut().set_name(name);
        self
    }

    /// Set the option type name of commit configuration.
    pub fn set_type<U: Infer>(self) -> UCommit<'a, S, U>
    where
        U::Val: RawValParser,
    {
        self.convert2infer::<U>()
    }

    /// Clear all the alias of commit configuration.
    pub fn clr_alias(mut self) -> Self {
        self.cfg_mut().clr_alias();
        self
    }

    /// Remove the given alias of commit configuration.
    pub fn rem_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.cfg_mut().rem_alias(alias);
        self
    }

    /// Add given alias into the commit configuration.
    pub fn add_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.cfg_mut().add_alias(alias);
        self
    }

    /// Set the option optional of commit configuration.
    pub fn set_force(mut self, force: bool) -> Self {
        self.cfg_mut().set_force(force);
        self
    }

    /// Set the option hint message of commit configuration.
    pub fn set_hint<T: Into<Str>>(mut self, hint: T) -> Self {
        self.cfg_mut().set_hint(hint);
        self
    }

    /// Set the option help message of commit configuration.
    pub fn set_help<T: Into<Str>>(mut self, help: T) -> Self {
        self.cfg_mut().set_help(help);
        self
    }

    /// Set the option value initiator.
    pub fn set_initializer(mut self, initializer: ValInitializer) -> Self {
        self.initializer = Some(initializer);
        self
    }

    /// Set the option value validator.
    pub fn set_validator<U: Infer>(self, validator: ValValidator<U::Val>) -> UCommit<'a, S, U>
    where
        U::Val: RawValParser,
    {
        self.convert2infer::<U>().set_validator(validator)
    }

    /// Set the option default value.
    pub fn set_value<U: Infer>(self, value: U::Val) -> UCommit<'a, S, U>
    where
        U::Val: Copy + RawValParser,
    {
        self.convert2infer::<U>().set_value(value)
    }

    /// Set the option default value.
    pub fn set_value_clone<U: Infer>(self, value: U::Val) -> UCommit<'a, S, U>
    where
        U::Val: Clone + RawValParser,
    {
        self.convert2infer::<U>().set_value_clone(value)
    }

    /// Set the option default value.
    pub fn set_values<U: Infer>(self, value: Vec<U::Val>) -> UCommit<'a, S, U>
    where
        U::Val: Clone + RawValParser,
    {
        self.convert2infer::<U>().set_values(value)
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        if let Some(commited) = self.commited {
            Ok(commited)
        } else {
            self.drop_commit = false;
            let info = std::mem::take(&mut self.info);
            let mut info = info.unwrap();
            let set = self.set.as_mut().unwrap();

            // Note !!
            // can't get type here, set the ValAccessor with fake type
            // fix it in option creator handler if `Config` has no typeid
            info.set_accessor(ValAccessor::from_option::<()>(
                self.initializer.take(),
                None,
            ));

            let default_ctor = crate::set::ctor_default_name();
            let _name = info.name().cloned();
            let ctor = info.ctor().unwrap_or(&default_ctor);
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

impl<'a, S> Drop for Commit<'a, S>
where
    S: Set,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop_commit && self.commited.is_none() {
            let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}
