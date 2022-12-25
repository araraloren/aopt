use std::fmt::Debug;
use tracing::trace;

use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::Assoc;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::set::Ctor;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetExt;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Create option using given configurations.
pub struct Commit<'a, S>
where
    S: Set,
    SetCfg<S>: ConfigValue + Default,
{
    info: SetCfg<S>,
    set: &'a mut S,
    commited: Option<Uid>,
    drop_commit: bool,
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
            set,
            info,
            commited: None,
            drop_commit: true,
        }
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

    /// Set the option value assoc type.
    pub fn set_assoc(mut self, assoc: Assoc) -> Self {
        self.info.set_assoc(assoc);
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
    pub fn set_type<T: Into<Str>>(mut self, type_name: T) -> Self {
        self.info.set_type(type_name);
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
    pub fn set_optional(mut self, optional: bool) -> Self {
        self.info.set_optional(optional);
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
    pub fn set_initiator(mut self, initiator: ValInitiator) -> Self {
        self.info.set_initiator(Some(initiator));
        self
    }

    /// Set the option value validator.
    pub fn set_validator(mut self, validator: ValValidator) -> Self {
        self.info.set_validator(Some(validator));
        self
    }

    /// Set the option default value.
    pub fn set_value<T: Clone + ErasedTy>(mut self, value: T) -> Self {
        self.info
            .set_initiator(Some(ValInitiator::with(vec![value])));
        self
    }

    /// Set the option default value.
    pub fn set_values<T: Clone + ErasedTy>(mut self, value: Vec<T>) -> Self {
        self.info.set_initiator(Some(ValInitiator::with(value)));
        self
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        if let Some(commited) = self.commited {
            Ok(commited)
        } else {
            let info = std::mem::take(&mut self.info);
            let type_name = info.gen_type()?;
            let name = info.name().cloned();
            let opt = self
                .set
                .ctor_mut(&type_name)?
                .new_with(info)
                .map_err(|e| e.into())?;
            let uid = self.set.insert(opt);

            trace!("Register a opt {:?} --> {}", name, uid);
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
