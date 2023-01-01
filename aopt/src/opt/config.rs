use crate::astr;
use crate::err::Error;
use crate::opt::Action;
use crate::opt::Assoc;
use crate::opt::Help;
use crate::opt::Index;
use crate::opt::Information;
use crate::opt::OptParser;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::Str;

pub trait Config {
    fn new<Parser>(parser: &Parser, pattern: Str) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information;
}

pub trait ConfigValue {
    /// The hint message used in usage of option.
    fn hint(&self) -> &Str;

    /// The help message of option.
    fn help(&self) -> &Str;

    /// The name of option.
    fn name(&self) -> Option<&Str>;

    /// The type name of option.
    fn r#type(&self) -> Option<&Str>;

    /// The index configuration of option.
    fn idx(&self) -> Option<&Index>;

    /// The alias name and prefix of option.
    fn alias(&self) -> Option<&Vec<Str>>;

    /// If the option is force required.
    fn force(&self) -> Option<bool>;

    /// Associated type of option.
    fn assoc(&self) -> Option<&Assoc>;

    /// Value action of option.
    fn action(&self) -> Option<&Action>;

    /// Value validator for option.
    fn validator(&self) -> Option<&ValValidator>;

    /// Value initiator for option
    fn initiator(&self) -> Option<&ValInitiator>;

    fn has_idx(&self) -> bool;

    fn has_name(&self) -> bool;

    fn has_type(&self) -> bool;

    fn has_hint(&self) -> bool;

    fn has_help(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_force(&self) -> bool;

    fn has_validator(&self) -> bool;

    fn has_initiator(&self) -> bool;

    fn set_idx(&mut self, index: Index) -> &mut Self;

    fn set_force(&mut self, force: bool) -> &mut Self;

    fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self;

    fn set_hint<S: Into<Str>>(&mut self, hint: S) -> &mut Self;

    fn set_help<S: Into<Str>>(&mut self, help: S) -> &mut Self;

    fn clr_alias(&mut self) -> &mut Self;

    fn add_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn rem_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn set_type<S: Into<Str>>(&mut self, type_name: S) -> &mut Self;

    fn set_assoc(&mut self, assoc: Assoc) -> &mut Self;

    fn set_action(&mut self, action: Action) -> &mut Self;

    fn set_initiator(&mut self, initiator: Option<ValInitiator>) -> &mut Self;

    fn set_validator(&mut self, validator: Option<ValValidator>) -> &mut Self;

    fn gen_name(&self) -> Result<Str, Error>;

    fn gen_type(&self) -> Result<Str, Error>;

    fn gen_idx(&self) -> Result<Index, Error>;

    fn gen_force(&self) -> Result<bool, Error>;

    fn gen_assoc(&self) -> Result<Assoc, Error>;

    fn gen_action(&self) -> Result<Action, Error>;

    fn gen_alias(&self) -> Result<Vec<Str>, Error>;

    fn gen_validator(&self) -> Result<ValValidator, Error>;

    fn gen_initiator(&self) -> Result<ValInitiator, Error>;

    fn gen_opt_help(&self) -> Result<Help, Error>;

    fn take_name(&mut self) -> Option<Str>;

    fn take_type(&mut self) -> Option<Str>;

    fn take_assoc(&mut self) -> Option<Assoc>;

    fn take_action(&mut self) -> Option<Action>;

    fn take_idx(&mut self) -> Option<Index>;

    fn take_force(&mut self) -> Option<bool>;

    fn take_alias(&mut self) -> Option<Vec<Str>>;

    fn take_opt_help(&mut self) -> Option<Help>;

    fn take_initiator(&mut self) -> Option<ValInitiator>;

    fn take_validator(&mut self) -> Option<ValValidator>;
}

/// Contain the information used for create option instance.
#[derive(Debug, Default)]
pub struct OptConfig {
    ty: Option<Str>,

    name: Option<Str>,

    force: Option<bool>,

    idx: Option<Index>,

    alias: Vec<Str>,

    help: Help,

    action: Option<Action>,

    assoc: Option<Assoc>,

    initiator: Option<ValInitiator>,

    valid: Option<ValValidator>,
}

impl OptConfig {
    pub fn with_idx(mut self, index: Index) -> Self {
        self.idx = Some(index);
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }

    pub fn with_name<S: Into<Str>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_type<S: Into<Str>>(mut self, type_name: S) -> Self {
        self.ty = Some(type_name.into());
        self
    }

    pub fn with_hint<S: Into<Str>>(mut self, hint: S) -> Self {
        self.help.set_hint(hint.into());
        self
    }

    pub fn with_help<S: Into<Str>>(mut self, help: S) -> Self {
        self.help.set_help(help.into());
        self
    }

    pub fn with_alias<S: Into<Str>>(mut self, alias: Vec<S>) -> Self {
        self.alias = alias.into_iter().map(|v| v.into()).collect();
        self
    }

    pub fn with_assoc(mut self, assoc: Option<Assoc>) -> Self {
        self.assoc = assoc;
        self
    }

    pub fn with_action(mut self, action: Option<Action>) -> Self {
        self.action = action;
        self
    }

    pub fn with_initiator(mut self, initiator: Option<ValInitiator>) -> Self {
        self.initiator = initiator;
        self
    }

    pub fn with_validator(mut self, validator: Option<ValValidator>) -> Self {
        self.valid = validator;
        self
    }

    pub fn raise_missing_error(&self, field: &str) -> Result<Error, Error> {
        Ok(Error::con_missing_field(
            &astr(field),
            self.name
                .as_ref()
                .ok_or_else(|| Error::raise_error("Option type name can't be empty"))?,
            self.ty
                .as_ref()
                .ok_or_else(|| Error::raise_error("Option name can't be empty"))?,
        ))
    }
}

impl Config for OptConfig {
    fn new<Parser>(parser: &Parser, pattern: Str) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information,
    {
        let mut output = parser.parse(pattern).map_err(|e| e.into())?;
        let mut ret = Self::default();

        if let Some(v) = output.take_name() {
            ret.set_name(v);
        }
        if let Some(v) = output.take_ty() {
            ret.set_type(v);
        }
        if let Some(v) = output.take_idx() {
            ret.set_idx(v);
        }
        if let Some(v) = output.take_force() {
            ret.set_force(v);
        }
        Ok(ret)
    }
}

impl ConfigValue for OptConfig {
    fn hint(&self) -> &Str {
        self.help.hint()
    }

    fn help(&self) -> &Str {
        self.help.help()
    }

    fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    fn r#type(&self) -> Option<&Str> {
        self.ty.as_ref()
    }

    fn idx(&self) -> Option<&Index> {
        self.idx.as_ref()
    }

    fn alias(&self) -> Option<&Vec<Str>> {
        Some(self.alias.as_ref())
    }

    fn force(&self) -> Option<bool> {
        self.force
    }

    fn assoc(&self) -> Option<&Assoc> {
        self.assoc.as_ref()
    }

    fn action(&self) -> Option<&Action> {
        self.action.as_ref()
    }

    fn validator(&self) -> Option<&ValValidator> {
        self.valid.as_ref()
    }

    fn initiator(&self) -> Option<&ValInitiator> {
        self.initiator.as_ref()
    }

    fn has_idx(&self) -> bool {
        self.idx.is_some()
    }

    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_type(&self) -> bool {
        self.ty.is_some()
    }

    fn has_hint(&self) -> bool {
        true
    }

    fn has_help(&self) -> bool {
        true
    }

    fn has_alias(&self) -> bool {
        !self.alias.is_empty()
    }

    fn has_force(&self) -> bool {
        self.force.is_some()
    }

    fn has_validator(&self) -> bool {
        self.valid.is_some()
    }

    fn has_initiator(&self) -> bool {
        self.initiator.is_some()
    }

    fn set_idx(&mut self, index: Index) -> &mut Self {
        self.idx = Some(index);
        self
    }

    fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = Some(force);
        self
    }

    fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    fn set_hint<S: Into<Str>>(&mut self, hint: S) -> &mut Self {
        self.help.set_hint(hint);
        self
    }

    fn set_help<S: Into<Str>>(&mut self, help: S) -> &mut Self {
        self.help.set_help(help);
        self
    }

    fn clr_alias(&mut self) -> &mut Self {
        self.alias.clear();
        self
    }

    fn add_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self {
        self.alias.push(alias.into());
        self
    }

    fn rem_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self {
        let alias = alias.into();

        for (index, value) in self.alias.iter().enumerate() {
            if value == &alias {
                self.alias.remove(index);
                break;
            }
        }
        self
    }

    fn set_type<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.ty = Some(type_name.into());
        self
    }

    fn set_assoc(&mut self, assoc: Assoc) -> &mut Self {
        self.assoc = Some(assoc);
        self
    }

    fn set_action(&mut self, action: Action) -> &mut Self {
        self.action = Some(action);
        self
    }

    fn set_initiator(&mut self, initiator: Option<ValInitiator>) -> &mut Self {
        self.initiator = initiator;
        self
    }

    fn set_validator(&mut self, validator: Option<ValValidator>) -> &mut Self {
        self.valid = validator;
        self
    }

    fn gen_name(&self) -> Result<Str, Error> {
        if let Some(name) = &self.name {
            return Ok(name.clone());
        }
        Err(self.raise_missing_error("name")?)
    }

    fn gen_type(&self) -> Result<Str, Error> {
        if let Some(type_name) = &self.ty {
            return Ok(type_name.clone());
        }
        Err(self.raise_missing_error("type name")?)
    }

    fn gen_idx(&self) -> Result<Index, Error> {
        if let Some(index) = self.idx.as_ref() {
            return Ok(index.clone());
        }
        Err(Error::con_missing_index(self.gen_name()?, self.gen_type()?))
    }

    fn gen_force(&self) -> Result<bool, Error> {
        if let Some(force) = self.force {
            return Ok(force);
        }
        Err(self.raise_missing_error("force")?)
    }

    fn gen_assoc(&self) -> Result<Assoc, Error> {
        if let Some(assoc) = self.assoc {
            return Ok(assoc);
        }
        Err(self.raise_missing_error("assoc")?)
    }

    fn gen_action(&self) -> Result<Action, Error> {
        if let Some(action) = self.action {
            return Ok(action);
        }
        Err(self.raise_missing_error("action")?)
    }

    fn gen_alias(&self) -> Result<Vec<Str>, Error> {
        return Ok(self.alias.clone());
    }

    fn gen_validator(&self) -> Result<ValValidator, Error> {
        Err(Error::raise_error(
            "Can not generate ValValidator, please take it",
        ))
    }

    fn gen_initiator(&self) -> Result<ValInitiator, Error> {
        Err(Error::raise_error(
            "Can not generate ValInitiator, please take it",
        ))
    }

    fn gen_opt_help(&self) -> Result<Help, Error> {
        let mut ret = self.help.clone();

        if ret.hint().is_empty() {
            let mut names = vec![String::default()];

            // add name
            names[0] += self.gen_name()?.as_ref();

            // add alias
            if let Some(alias_vec) = self.alias() {
                for alias in alias_vec {
                    names.push(format!("{}", alias));
                }
            }
            // sort name by len
            names.sort_by_key(|v| v.len());

            if let Some(index) = &self.idx {
                let index_string = index.to_help();

                // add index string
                if index_string.is_empty() {
                    ret.set_hint(names.join(","));
                } else {
                    ret.set_hint(format!("{}@{}", names.join(","), index_string));
                }
            } else {
                ret.set_hint(names.join(","));
            }
        }
        Ok(ret)
    }

    fn take_name(&mut self) -> Option<Str> {
        self.name.take()
    }

    fn take_type(&mut self) -> Option<Str> {
        self.ty.take()
    }

    fn take_assoc(&mut self) -> Option<Assoc> {
        self.assoc.take()
    }

    fn take_action(&mut self) -> Option<Action> {
        self.action.take()
    }

    fn take_idx(&mut self) -> Option<Index> {
        self.idx.take()
    }

    fn take_force(&mut self) -> Option<bool> {
        self.force.take()
    }

    fn take_alias(&mut self) -> Option<Vec<Str>> {
        Some(std::mem::take(&mut self.alias))
    }

    fn take_opt_help(&mut self) -> Option<Help> {
        Some(std::mem::take(&mut self.help))
    }

    fn take_initiator(&mut self) -> Option<ValInitiator> {
        self.initiator.take()
    }

    fn take_validator(&mut self) -> Option<ValValidator> {
        self.valid.take()
    }
}
