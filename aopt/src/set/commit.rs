use std::fmt::Debug;

use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Creator;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptIndex;
use crate::opt::OptParser;
use crate::opt::ValAction;
use crate::opt::ValAssoc;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::set::OptSet;
use crate::set::Pre;
use crate::set::Set;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Create option using given configurations.
pub struct Commit<'a, Parser, Ctor>
where
    Ctor: Creator,
    Parser: OptParser,
    Ctor::Config: Config + ConfigValue,
{
    info: Ctor::Config,
    set: &'a mut OptSet<Parser, Ctor>,
}

impl<'a, Parser, Ctor> Debug for Commit<'a, Parser, Ctor>
where
    Ctor::Opt: Debug,
    Ctor: Creator + Debug,
    Parser: OptParser + Debug,
    Ctor::Config: Config + ConfigValue + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Commit")
            .field("info", &self.info)
            .field("set", &self.set)
            .finish()
    }
}

impl<'a, Parser, Ctor> Commit<'a, Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser + Pre,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    pub fn new(set: &'a mut OptSet<Parser, Ctor>, info: Ctor::Config) -> Self {
        Self { set, info }
    }

    pub fn cfg(&self) -> &Ctor::Config {
        &self.info
    }

    pub fn cfg_mut(&mut self) -> &mut Ctor::Config {
        &mut self.info
    }

    /// Set the option index of commit configuration.
    pub fn set_idx(&mut self, index: OptIndex) -> &mut Self {
        self.info.set_idx(index);
        self
    }

    /// Set the option value assoc type.
    pub fn set_assoc(&mut self, assoc: ValAssoc) -> &mut Self {
        self.info.set_assoc(assoc);
        self
    }

    /// Set the option value action.
    pub fn set_action(&mut self, action: ValAction) -> &mut Self {
        self.info.set_action(action);
        self
    }

    /// Set the option name of commit configuration.
    pub fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of commit configuration.
    pub fn set_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of commit configuration.
    pub fn set_type<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.info.set_type(type_name);
        self
    }

    /// Clear all the alias of commit configuration.
    pub fn clr_alias(&mut self) -> &mut Self {
        self.info.clr_alias();
        self
    }

    /// Remove the given alias of commit configuration.
    pub fn rem_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self {
        self.info.rem_alias(alias);
        self
    }

    /// Add given alias into the commit configuration.
    pub fn add_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self {
        self.info.add_alias(alias);
        self
    }

    /// Set the option optional of commit configuration.
    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    /// Set the option hint message of commit configuration.
    pub fn set_hint<S: Into<Str>>(&mut self, hint: S) -> &mut Self {
        self.info.set_hint(hint);
        self
    }

    /// Set the option help message of commit configuration.
    pub fn set_help<S: Into<Str>>(&mut self, help: S) -> &mut Self {
        self.info.set_help(help);
        self
    }

    /// Set the option value initiator.
    pub fn set_initiator(&mut self, initiator: ValInitiator) -> &mut Self {
        self.info.set_initiator(Some(initiator));
        self
    }

    /// Set the option value validator.
    pub fn set_validator(&mut self, validator: ValValidator) -> &mut Self {
        self.info.set_validator(Some(validator));
        self
    }

    /// Set the option deactivate style of commit configuration.
    pub fn set_deactivate(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate(deactivate_style);
        self
    }

    /// Set the option default value.
    pub fn set_value<T: Clone + 'static>(&mut self, value: T) -> &mut Self {
        self.info
            .set_initiator(Some(ValInitiator::with(vec![value])));
        self
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Creator`].
    /// And add it to referenced [`OptSet`], return the new option [`Uid`].
    pub fn run(&mut self) -> Result<Uid, Error> {
        let info = std::mem::take(&mut self.info);
        let type_name = info.gen_type()?;
        let opt = self
            .set
            .creator(&type_name)
            .as_mut()
            .ok_or_else(|| Error::con_unsupport_option_type(type_name))?
            .new_with(info)
            .map_err(|e| e.into())?;

        Ok(self.set.insert(opt))
    }
}
