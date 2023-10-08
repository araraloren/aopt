use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::err::Error;
use crate::opt::Action;
use crate::opt::Help;
use crate::opt::Index;
use crate::opt::Information;
use crate::opt::OptParser;
use crate::typeid;
use crate::value::Placeholder;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::Str;

use super::Style;

pub trait ConfigBuild {
    type Infer;

    type Config;

    fn describe(&self) -> &str;

    fn build<Parser>(self, parser: &Parser) -> Result<Self::Config, Error>
    where
        Parser: OptParser,
        Parser::Output: Information;
}

pub trait ConfigValue {
    /// The hint message used in usage of option.
    fn hint(&self) -> &Str;

    /// The help message of option.
    fn help(&self) -> &Str;

    /// The style support by current option.
    fn style(&self) -> Option<&Vec<Style>>;

    /// The creator name of option.
    fn ctor(&self) -> Option<&Str>;

    /// The name of option.
    fn name(&self) -> Option<&Str>;

    /// The type name of option.
    fn r#type(&self) -> Option<&TypeId>;

    /// The index configuration of option.
    fn index(&self) -> Option<&Index>;

    /// The alias name and prefix of option.
    fn alias(&self) -> Option<&Vec<Str>>;

    /// If the option is force required.
    fn force(&self) -> Option<bool>;

    /// Value action of option.
    fn action(&self) -> Option<&Action>;

    /// Value validator for option.
    fn storer(&self) -> Option<&ValStorer>;

    /// Value initializer for option.
    fn initializer(&self) -> Option<&ValInitializer>;

    fn ignore_name(&self) -> bool;

    fn ignore_alias(&self) -> bool;

    fn ignore_index(&self) -> bool;

    fn has_infer(&self) -> bool;

    fn has_index(&self) -> bool;

    fn has_ctor(&self) -> bool;

    fn has_name(&self) -> bool;

    fn has_type(&self) -> bool;

    fn has_hint(&self) -> bool;

    fn has_help(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_force(&self) -> bool;

    fn has_style(&self) -> bool;

    fn has_action(&self) -> bool;

    fn has_storer(&self) -> bool;

    fn has_initializer(&self) -> bool;

    fn set_ignore_name(&mut self, ignore_name: bool) -> &mut Self;

    fn set_ignore_alias(&mut self, ignore_alias: bool) -> &mut Self;

    fn set_ignore_index(&mut self, ignore_index: bool) -> &mut Self;

    fn set_infer(&mut self, infered: bool) -> &mut Self;

    fn set_index(&mut self, index: Index) -> &mut Self;

    fn set_force(&mut self, force: bool) -> &mut Self;

    fn set_ctor(&mut self, ctor: impl Into<Str>) -> &mut Self;

    fn set_name(&mut self, name: impl Into<Str>) -> &mut Self;

    fn set_hint(&mut self, hint: impl Into<Str>) -> &mut Self;

    fn set_help(&mut self, help: impl Into<Str>) -> &mut Self;

    fn set_style(&mut self, styles: Vec<Style>) -> &mut Self;

    fn clr_alias(&mut self) -> &mut Self;

    fn add_alias(&mut self, alias: impl Into<Str>) -> &mut Self;

    fn rem_alias(&mut self, alias: impl Into<Str>) -> &mut Self;

    fn set_type<T: 'static>(&mut self) -> &mut Self;

    fn set_type_id(&mut self, type_id: TypeId) -> &mut Self;

    fn set_action(&mut self, action: Action) -> &mut Self;

    fn set_storer(&mut self, storer: ValStorer) -> &mut Self;

    fn set_initializer(&mut self, initializer: ValInitializer) -> &mut Self;
}

/// Contain the information used for create option instance.
#[derive(Debug, Default)]
pub struct OptConfig {
    ctor: Option<Str>,

    r#type: Option<TypeId>,

    name: Option<Str>,

    force: Option<bool>,

    index: Option<Index>,

    alias: Vec<Str>,

    help: Help,

    action: Option<Action>,

    storer: Option<ValStorer>,

    initializer: Option<ValInitializer>,

    ignore_name: bool,

    ignore_alias: bool,

    ignore_index: bool,

    infered: bool,

    styles: Option<Vec<Style>>,
}

impl OptConfig {
    pub fn with_index(mut self, index: Index) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }

    pub fn with_ctor(mut self, ctor: impl Into<Str>) -> Self {
        self.ctor = Some(ctor.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<Str>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_type<T: 'static>(mut self) -> Self {
        self.r#type = Some(typeid::<T>());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<Str>) -> Self {
        self.help.set_hint(hint.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<Str>) -> Self {
        self.help.set_help(help.into());
        self
    }

    pub fn with_alias(mut self, alias: Vec<impl Into<Str>>) -> Self {
        self.alias = alias.into_iter().map(|v| v.into()).collect();
        self
    }

    pub fn with_styles(mut self, styles: Vec<Style>) -> Self {
        self.styles = Some(styles);
        self
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_storer(mut self, storer: ValStorer) -> Self {
        self.storer = Some(storer);
        self
    }

    pub fn with_ignore_alias(mut self, ignore_alias: bool) -> Self {
        self.ignore_alias = ignore_alias;
        self
    }

    pub fn with_ignore_index(mut self, ignore_index: bool) -> Self {
        self.ignore_index = ignore_index;
        self
    }

    pub fn with_ignore_name(mut self, ignore_name: bool) -> Self {
        self.ignore_name = ignore_name;
        self
    }

    pub fn with_initializer(mut self, initializer: ValInitializer) -> Self {
        self.initializer = Some(initializer);
        self
    }

    pub fn take_alias(&mut self) -> Vec<Str> {
        std::mem::take(&mut self.alias)
    }

    pub fn take_storer(&mut self) -> Option<ValStorer> {
        self.storer.take()
    }

    pub fn take_initializer(&mut self) -> Option<ValInitializer> {
        self.initializer.take()
    }

    pub fn gen_name(&self) -> Result<Str, Error> {
        Ok(self
            .name
            .as_ref()
            .ok_or_else(|| {
                crate::raise_error!("Incomplete option configuration: missing option name")
            })?
            .clone())
    }

    pub fn gen_type(&mut self) -> Result<TypeId, Error> {
        self.r#type.take().ok_or_else(|| {
            crate::raise_error!("Incomplete option configuration: missing option value type")
        })
    }

    pub fn gen_storer(&mut self) -> Result<ValStorer, Error> {
        self.storer.take().ok_or_else(|| {
            crate::raise_error!("Incomplete option configuration: missing ValStorer")
        })
    }

    pub fn gen_initializer(&mut self) -> Result<ValInitializer, Error> {
        self.initializer.take().ok_or_else(|| {
            crate::raise_error!("Incomplete option configuration: missing ValInitializer")
        })
    }

    pub fn gen_styles(&mut self) -> Result<Vec<Style>, Error> {
        self.styles
            .take()
            .ok_or_else(|| crate::raise_error!("Incomplete option configuration: missing Style"))
    }

    pub fn gen_opt_help(&self) -> Result<Help, Error> {
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

            if let Some(index) = &self.index {
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
}

impl ConfigValue for OptConfig {
    fn hint(&self) -> &Str {
        self.help.hint()
    }

    fn help(&self) -> &Str {
        self.help.help()
    }

    fn style(&self) -> Option<&Vec<Style>> {
        self.styles.as_ref()
    }

    fn ctor(&self) -> Option<&Str> {
        self.ctor.as_ref()
    }

    fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    fn r#type(&self) -> Option<&TypeId> {
        self.r#type.as_ref()
    }

    fn index(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn alias(&self) -> Option<&Vec<Str>> {
        Some(self.alias.as_ref())
    }

    fn force(&self) -> Option<bool> {
        self.force
    }

    fn action(&self) -> Option<&Action> {
        self.action.as_ref()
    }

    fn storer(&self) -> Option<&ValStorer> {
        self.storer.as_ref()
    }

    fn initializer(&self) -> Option<&ValInitializer> {
        self.initializer.as_ref()
    }

    fn ignore_name(&self) -> bool {
        self.ignore_name
    }

    fn ignore_alias(&self) -> bool {
        self.ignore_alias
    }

    fn ignore_index(&self) -> bool {
        self.ignore_index
    }

    fn has_infer(&self) -> bool {
        self.infered
    }

    fn has_index(&self) -> bool {
        self.index.is_some()
    }

    fn has_ctor(&self) -> bool {
        self.ctor.is_some()
    }

    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_type(&self) -> bool {
        self.r#type.is_some()
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

    fn has_style(&self) -> bool {
        self.styles.is_some()
    }

    fn has_action(&self) -> bool {
        self.action.is_some()
    }

    fn has_storer(&self) -> bool {
        self.storer.is_some()
    }

    fn has_initializer(&self) -> bool {
        self.initializer.is_some()
    }

    fn set_ignore_name(&mut self, ignore_name: bool) -> &mut Self {
        self.ignore_name = ignore_name;
        self
    }

    fn set_ignore_alias(&mut self, ignore_alias: bool) -> &mut Self {
        self.ignore_alias = ignore_alias;
        self
    }

    fn set_ignore_index(&mut self, ignore_index: bool) -> &mut Self {
        self.ignore_index = ignore_index;
        self
    }

    fn set_infer(&mut self, fix_infer: bool) -> &mut Self {
        self.infered = fix_infer;
        self
    }

    fn set_index(&mut self, index: Index) -> &mut Self {
        self.index = Some(index);
        self
    }

    fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = Some(force);
        self
    }

    fn set_ctor(&mut self, ctor: impl Into<Str>) -> &mut Self {
        self.ctor = Some(ctor.into());
        self
    }

    fn set_name(&mut self, name: impl Into<Str>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    fn set_hint(&mut self, hint: impl Into<Str>) -> &mut Self {
        self.help.set_hint(hint);
        self
    }

    fn set_help(&mut self, help: impl Into<Str>) -> &mut Self {
        self.help.set_help(help);
        self
    }

    fn set_style(&mut self, styles: Vec<Style>) -> &mut Self {
        self.styles = Some(styles);
        self
    }

    fn clr_alias(&mut self) -> &mut Self {
        self.alias.clear();
        self
    }

    fn add_alias(&mut self, alias: impl Into<Str>) -> &mut Self {
        self.alias.push(alias.into());
        self
    }

    fn rem_alias(&mut self, alias: impl Into<Str>) -> &mut Self {
        let alias = alias.into();

        for (index, value) in self.alias.iter().enumerate() {
            if value == &alias {
                self.alias.remove(index);
                break;
            }
        }
        self
    }

    fn set_type<T: 'static>(&mut self) -> &mut Self {
        self.r#type = Some(typeid::<T>());
        self
    }

    fn set_type_id(&mut self, type_id: TypeId) -> &mut Self {
        self.r#type = Some(type_id);
        self
    }

    fn set_action(&mut self, action: Action) -> &mut Self {
        self.action = Some(action);
        self
    }

    fn set_storer(&mut self, storer: ValStorer) -> &mut Self {
        self.storer = Some(storer);
        self
    }

    fn set_initializer(&mut self, initializer: ValInitializer) -> &mut Self {
        self.initializer = Some(initializer);
        self
    }
}

/// Contain the information used for create option instance.
pub struct ConfigBuilder<I> {
    creator: Option<Str>,

    ctor: Option<Str>,

    r#type: Option<TypeId>,

    name: Option<Str>,

    force: Option<bool>,

    index: Option<Index>,

    alias: Option<Vec<Str>>,

    help: Option<Help>,

    action: Option<Action>,

    storer: Option<ValStorer>,

    initializer: Option<ValInitializer>,

    ignore_name: Option<bool>,

    ignore_alias: Option<bool>,

    ignore_index: Option<bool>,

    styles: Option<Vec<Style>>,

    marker: PhantomData<I>,
}

impl<I> Default for ConfigBuilder<I> {
    fn default() -> Self {
        Self {
            creator: Default::default(),
            ctor: Default::default(),
            r#type: Default::default(),
            name: Default::default(),
            force: Default::default(),
            index: Default::default(),
            alias: Default::default(),
            help: Default::default(),
            action: Default::default(),
            storer: Default::default(),
            initializer: Default::default(),
            ignore_name: Default::default(),
            ignore_alias: Default::default(),
            ignore_index: Default::default(),
            styles: Default::default(),
            marker: Default::default(),
        }
    }
}

impl<I> Debug for ConfigBuilder<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigBuilder")
            .field("creator", &self.creator)
            .field("ctor", &self.ctor)
            .field("r#type", &self.r#type)
            .field("name", &self.name)
            .field("force", &self.force)
            .field("index", &self.index)
            .field("alias", &self.alias)
            .field("help", &self.help)
            .field("action", &self.action)
            .field("storer", &self.storer)
            .field("initializer", &self.initializer)
            .field("ignore_name", &self.ignore_name)
            .field("ignore_alias", &self.ignore_alias)
            .field("ignore_index", &self.ignore_index)
            .field("styles", &self.styles)
            .field("marker", &self.marker)
            .finish()
    }
}

impl<I> ConfigBuilder<I> {
    pub fn new(creator: Option<Str>) -> Self {
        Self {
            creator,
            ..Default::default()
        }
    }

    pub fn with_index(mut self, index: Index) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }

    pub fn with_ctor(mut self, ctor: impl Into<Str>) -> Self {
        self.ctor = Some(ctor.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<Str>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_type<T: 'static>(mut self) -> Self {
        self.r#type = Some(typeid::<T>());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<Str>) -> Self {
        self.help
            .get_or_insert(Help::default())
            .set_hint(hint.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<Str>) -> Self {
        self.help
            .get_or_insert(Help::default())
            .set_help(help.into());
        self
    }

    pub fn with_alias(mut self, alias: Vec<impl Into<Str>>) -> Self {
        self.alias = Some(alias.into_iter().map(|v| v.into()).collect());
        self
    }

    pub fn with_styles(mut self, styles: Vec<Style>) -> Self {
        self.styles = Some(styles);
        self
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_storer(mut self, storer: ValStorer) -> Self {
        self.storer = Some(storer);
        self
    }

    pub fn with_ignore_alias(mut self, ignore_alias: bool) -> Self {
        self.ignore_alias = Some(ignore_alias);
        self
    }

    pub fn with_ignore_index(mut self, ignore_index: bool) -> Self {
        self.ignore_index = Some(ignore_index);
        self
    }

    pub fn with_ignore_name(mut self, ignore_name: bool) -> Self {
        self.ignore_name = Some(ignore_name);
        self
    }

    pub fn with_initializer(mut self, initializer: ValInitializer) -> Self {
        self.initializer = Some(initializer);
        self
    }
}

impl<I> From<Str> for ConfigBuilder<I> {
    fn from(value: Str) -> Self {
        Self::new(Some(value))
    }
}

impl ConfigBuild for Str {
    type Infer = Placeholder;

    type Config = OptConfig;

    fn describe(&self) -> &str {
        self.as_str()
    }

    fn build<Parser>(self, parser: &Parser) -> Result<Self::Config, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information,
    {
        let mut output = parser.parse_opt(self).map_err(|e| e.into())?;
        let mut ret = Self::Config::default();

        if let Some(v) = output.take_name() {
            ret.set_name(v);
        }
        if let Some(v) = output.take_index() {
            ret.set_index(v);
        }
        if let Some(v) = output.take_force() {
            ret.set_force(v);
        }
        if let Some(v) = output.take_help() {
            ret.set_help(v);
        }
        if let Some(v) = output.take_ctor() {
            ret.set_ctor(v);
        }
        if let Some(v) = output.take_alias() {
            for item in v {
                ret.add_alias(item);
            }
        }
        Ok(ret)
    }
}

impl ConfigBuild for &'_ str {
    type Infer = Placeholder;

    type Config = OptConfig;

    fn describe(&self) -> &str {
        self
    }

    fn build<Parser>(self, parser: &Parser) -> Result<Self::Config, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information,
    {
        ConfigBuild::build(Str::from(self), parser)
    }
}

impl ConfigBuild for String {
    type Infer = Placeholder;

    type Config = OptConfig;

    fn describe(&self) -> &str {
        self.as_str()
    }

    fn build<Parser>(self, parser: &Parser) -> Result<Self::Config, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information,
    {
        ConfigBuild::build(Str::from(self), parser)
    }
}

impl<I> ConfigBuild for ConfigBuilder<I> {
    type Infer = I;

    type Config = OptConfig;

    fn describe(&self) -> &str {
        self.name.as_ref().map(|v|v.as_str()).unwrap_or("Empty name")
    }

    fn build<Parser>(self, parser: &Parser) -> Result<Self::Config, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information,
    {
        let mut ret = if let Some(creator) = self.creator {
            creator.build(parser)?
        } else {
            Self::Config::default()
        };

        if let Some(ctor) = self.ctor {
            ret.with_ctor(ctor);
        }
        if let Some(r#type) = self.r#type {
            ret.set_type_id(r#type);
        }
        if let Some(name) = self.name {
            ret.with_name(name);
        }
        if let Some(force) = self.force {
            ret.with_force(force);
        }
        if let Some(index) = self.index {
            ret.with_index(index);
        }
        if let Some(alias) = self.alias {
            ret.with_alias(alias);
        }
        if let Some(help) = self.help {
            ret.with_help(help.help());
            ret.with_hint(help.hint());
        }
        if let Some(action) = self.action {
            ret.with_action(action);
        }
        if let Some(storer) = self.storer {
            ret.with_storer(storer);
        }
        if let Some(initializer) = self.initializer {
            ret.with_initializer(initializer);
        }
        if let Some(ignore_name) = self.ignore_name {
            ret.with_ignore_name(ignore_name);
        }
        if let Some(ignore_alias) = self.ignore_alias {
            ret.with_ignore_alias(ignore_alias);
        }
        if let Some(ignore_index) = self.ignore_index {
            ret.with_ignore_index(ignore_index);
        }
        if let Some(styles) = self.styles {
            ret.with_styles(styles);
        }
        Ok(ret)
    }
}

impl ConfigBuild for OptConfig {
    type Infer = Placeholder;

    type Config = OptConfig;

    fn describe(&self) -> &str {
        self.name.as_ref().map(|v|v.as_str()).unwrap_or("Empty name")
    }

    fn build<Parser>(self, parser: &Parser) -> Result<Self::Config, Error>
    where
        Parser: OptParser,
        Parser::Output: Information {
        Ok(self)
    }
}