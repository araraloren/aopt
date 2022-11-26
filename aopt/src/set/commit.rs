use std::fmt::Debug;
use tracing::trace;

use crate::opt::Action;
use crate::opt::Assoc;
use crate::opt::ConfigValue;
use crate::opt::Creator;
use crate::opt::Index;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::set::SetExt;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Create option using given configurations.
pub struct Commit<'a, Set>
where
    Set: crate::set::Set,
    <Set::Ctor as Creator>::Config: ConfigValue + Default,
{
    info: <Set::Ctor as Creator>::Config,
    set: &'a mut Set,
    commited: Option<Uid>,
}

impl<'a, Set> Debug for Commit<'a, Set>
where
    Set: crate::set::Set + Debug,
    <Set::Ctor as Creator>::Config: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commit")
            .field("info", &self.info)
            .field("set", &self.set)
            .finish()
    }
}

impl<'a, Set> Commit<'a, Set>
where
    Set: crate::set::Set,
    <Set::Ctor as Creator>::Config: ConfigValue + Default,
{
    pub fn new(set: &'a mut Set, info: <Set::Ctor as Creator>::Config) -> Self {
        Self {
            set,
            info,
            commited: None,
        }
    }

    pub fn cfg(&self) -> &<Set::Ctor as Creator>::Config {
        &self.info
    }

    pub fn cfg_mut(&mut self) -> &mut <Set::Ctor as Creator>::Config {
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
    pub fn set_name<S: Into<Str>>(mut self, name: S) -> Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of commit configuration.
    pub fn set_prefix<S: Into<Str>>(mut self, prefix: S) -> Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of commit configuration.
    pub fn set_type<S: Into<Str>>(mut self, type_name: S) -> Self {
        self.info.set_type(type_name);
        self
    }

    /// Clear all the alias of commit configuration.
    pub fn clr_alias(mut self) -> Self {
        self.info.clr_alias();
        self
    }

    /// Remove the given alias of commit configuration.
    pub fn rem_alias<S: Into<Str>>(mut self, alias: S) -> Self {
        self.info.rem_alias(alias);
        self
    }

    /// Add given alias into the commit configuration.
    pub fn add_alias<S: Into<Str>>(mut self, alias: S) -> Self {
        self.info.add_alias(alias);
        self
    }

    /// Set the option optional of commit configuration.
    pub fn set_optional(mut self, optional: bool) -> Self {
        self.info.set_optional(optional);
        self
    }

    /// Set the option hint message of commit configuration.
    pub fn set_hint<S: Into<Str>>(mut self, hint: S) -> Self {
        self.info.set_hint(hint);
        self
    }

    /// Set the option help message of commit configuration.
    pub fn set_help<S: Into<Str>>(mut self, help: S) -> Self {
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

    /// Set the option deactivate style of commit configuration.
    pub fn set_deactivate(mut self, deactivate_style: bool) -> Self {
        self.info.set_deactivate(deactivate_style);
        self
    }

    /// Set the option default value.
    pub fn set_value<T: Clone + 'static>(mut self, value: T) -> Self {
        self.info
            .set_initiator(Some(ValInitiator::with(vec![value])));
        self
    }

    pub fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
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
    /// It create an option using given type [`Creator`].
    /// And add it to referenced [`Set`](crate::set::Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.run_and_commit_the_change()
    }
}

impl<'a, Set> Drop for Commit<'a, Set>
where
    Set: crate::set::Set,
    <Set::Ctor as Creator>::Config: ConfigValue + Default,
{
    fn drop(&mut self) {
        let error = "Error when commit the option in Commit::Drop, call `run` get the Result";

        self.run_and_commit_the_change().expect(error);
    }
}
