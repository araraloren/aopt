use std::fmt::Debug;

use super::Prefixed;
use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Creator;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptCallback;
use crate::opt::OptIndex;
use crate::opt::OptParser;
use crate::set::OptSet;
use crate::set::Set;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Create option using given configurations.
pub struct Commit<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T>,
    Parser: OptParser,
    Ctor::Config: Config + ConfigValue,
{
    info: Ctor::Config,
    set: &'a mut OptSet<T, Parser, Ctor>,
}

impl<'a, T, Parser, Ctor> Debug for Commit<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T> + Debug,
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

impl<'a, T, Parser, Ctor> Commit<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T>,
    Parser: OptParser + Prefixed,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    pub fn new(set: &'a mut OptSet<T, Parser, Ctor>, info: Ctor::Config) -> Self {
        Self { set, info }
    }

    pub fn config(&self) -> &Ctor::Config {
        &self.info
    }

    pub fn config_mut(&mut self) -> &mut Ctor::Config {
        &mut self.info
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
    pub fn set_type_name<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.info.set_type_name(type_name);
        self
    }

    /// Set the option index of commit configuration.
    pub fn set_index(&mut self, index: OptIndex) -> &mut Self {
        self.info.set_index(index);
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

    /// Set the option deactivate style of commit configuration.
    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    /// Set the option callback of commit configuration.
    pub fn set_callback<U>(&mut self, callback: OptCallback<U>) -> &mut Self
    where
        T: 'static,
    {
        self.info.set_callback(callback);
        self
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Creator`].
    /// And add it to referenced [`OptSet`], return the new option [`Uid`].
    pub fn run(&mut self) -> Result<Uid, Error> {
        let info = std::mem::take(&mut self.info);
        let type_name = info.gen_type_name()?;
        let opt = self
            .set
            .get_creator(&type_name)
            .as_mut()
            .ok_or_else(|| Error::con_unsupport_option_type(type_name))?
            .create_with(info)
            .map_err(|e| e.into())?;

        Ok(self.set.insert(opt))
    }
}
