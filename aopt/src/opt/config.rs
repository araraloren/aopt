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
    fn help(&self) -> Option<&Help>;

    /// The style support by current option.
    fn style(&self) -> Option<&Vec<Style>>;

    /// The creator name of option.
    fn ctor(&self) -> Option<&str>;

    /// The name of option.
    fn name(&self) -> Option<&str>;

    /// The type name of option.
    fn r#type(&self) -> Option<&TypeId>;

    /// The index configuration of option.
    fn index(&self) -> Option<&Index>;

    /// The alias name and prefix of option.
    fn alias(&self) -> Option<&Vec<String>>;

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

    fn has_index(&self) -> bool;

    fn has_ctor(&self) -> bool;

    fn has_name(&self) -> bool;

    fn has_type(&self) -> bool;

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

    fn set_index(&mut self, index: Index) -> &mut Self;

    fn set_force(&mut self, force: bool) -> &mut Self;

    fn set_ctor(&mut self, ctor: impl Into<String>) -> &mut Self;

    fn set_name(&mut self, name: impl Into<String>) -> &mut Self;

    fn set_help(&mut self, help: impl Into<Help>) -> &mut Self;

    fn set_styles(&mut self, styles: Vec<Style>) -> &mut Self;

    fn clr_alias(&mut self) -> &mut Self;

    fn add_alias(&mut self, alias: impl Into<String>) -> &mut Self;

    fn rem_alias(&mut self, alias: impl Into<String>) -> &mut Self;

    fn set_alias(&mut self, alias: Vec<impl Into<String>>) -> &mut Self;

    fn set_type<T: 'static>(&mut self) -> &mut Self;

    fn set_type_id(&mut self, type_id: TypeId) -> &mut Self;

    fn set_action(&mut self, action: Action) -> &mut Self;

    fn set_storer(&mut self, storer: ValStorer) -> &mut Self;

    fn set_initializer(&mut self, initializer: ValInitializer) -> &mut Self;

    fn take_index(&mut self) -> Option<Index>;

    fn take_ctor(&mut self) -> Option<String>;

    fn take_name(&mut self) -> Option<String>;

    fn take_type(&mut self) -> Option<TypeId>;

    fn take_help(&mut self) -> Option<Help>;

    fn take_alias(&mut self) -> Option<Vec<String>>;

    fn take_styles(&mut self) -> Option<Vec<Style>>;

    fn take_action(&mut self) -> Option<Action>;

    fn take_storer(&mut self) -> Option<ValStorer>;

    fn take_initializer(&mut self) -> Option<ValInitializer>;
}

/// Contain the information used for create option instance.
#[derive(Debug, Default)]
pub struct OptConfig {
    ctor: Option<String>,

    r#type: Option<TypeId>,

    name: Option<String>,

    force: Option<bool>,

    index: Option<Index>,

    alias: Vec<String>,

    help: Option<Help>,

    action: Option<Action>,

    storer: Option<ValStorer>,

    initializer: Option<ValInitializer>,

    ignore_name: bool,

    ignore_alias: bool,

    ignore_index: bool,

    styles: Option<Vec<Style>>,
}

impl ConfigValue for OptConfig {
    fn help(&self) -> Option<&Help> {
        self.help.as_ref()
    }

    fn style(&self) -> Option<&Vec<Style>> {
        self.styles.as_ref()
    }

    fn ctor(&self) -> Option<&str> {
        self.ctor.as_deref()
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn r#type(&self) -> Option<&TypeId> {
        self.r#type.as_ref()
    }

    fn index(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn alias(&self) -> Option<&Vec<String>> {
        Some(&self.alias)
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

    fn has_help(&self) -> bool {
        self.help.is_some()
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

    fn set_index(&mut self, index: Index) -> &mut Self {
        self.index = Some(index);
        self
    }

    fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = Some(force);
        self
    }

    fn set_ctor(&mut self, ctor: impl Into<String>) -> &mut Self {
        self.ctor = Some(ctor.into());
        self
    }

    fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    fn set_help(&mut self, help: impl Into<Help>) -> &mut Self {
        self.help = Some(help.into());
        self
    }

    fn set_styles(&mut self, styles: Vec<Style>) -> &mut Self {
        self.styles = Some(styles);
        self
    }

    fn clr_alias(&mut self) -> &mut Self {
        self.alias.clear();
        self
    }

    fn add_alias(&mut self, alias: impl Into<String>) -> &mut Self {
        self.alias.push(alias.into());
        self
    }

    fn rem_alias(&mut self, alias: impl Into<String>) -> &mut Self {
        let alias = alias.into();

        for (index, value) in self.alias.iter().enumerate() {
            if value == &alias {
                self.alias.remove(index);
                break;
            }
        }
        self
    }

    fn set_alias(&mut self, alias: Vec<impl Into<String>>) -> &mut Self {
        self.alias = alias.into_iter().map(Into::into).collect();
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

    fn take_index(&mut self) -> Option<Index> {
        self.index.take()
    }

    fn take_ctor(&mut self) -> Option<String> {
        self.ctor.take()
    }

    fn take_name(&mut self) -> Option<String> {
        self.name.take()
    }

    fn take_type(&mut self) -> Option<TypeId> {
        self.r#type.take()
    }

    fn take_help(&mut self) -> Option<Help> {
        self.help.take()
    }

    fn take_alias(&mut self) -> Option<Vec<String>> {
        Some(std::mem::take(&mut self.alias))
    }

    fn take_styles(&mut self) -> Option<Vec<Style>> {
        self.styles.take()
    }

    fn take_action(&mut self) -> Option<Action> {
        self.action.take()
    }

    fn take_storer(&mut self) -> Option<ValStorer> {
        self.storer.take()
    }

    fn take_initializer(&mut self) -> Option<ValInitializer> {
        self.initializer.take()
    }
}

pub struct ConfigBuilder<I> {
    creator: Option<String>,

    ctor: Option<String>,

    name: Option<String>,

    force: Option<bool>,

    index: Option<Index>,

    alias: Option<Vec<String>>,

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

impl<I: 'static> ConfigBuilder<I> {
    pub fn with_creator(mut self, init: impl Into<String>) -> Self {
        self.creator = Some(init.into());
        self
    }

    pub fn with_index(mut self, index: Index) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }

    pub fn with_ctor(mut self, ctor: impl Into<String>) -> Self {
        self.ctor = Some(ctor.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<Help>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_alias(mut self, alias: Vec<impl Into<String>>) -> Self {
        self.alias = Some(alias.into_iter().map(Into::into).collect());
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

    pub fn take_creator(&mut self) -> Option<String> {
        self.creator.take()
    }

    pub fn take_index(&mut self) -> Option<Index> {
        self.index.take()
    }

    pub fn take_ctor(&mut self) -> Option<String> {
        self.ctor.take()
    }

    pub fn take_name(&mut self) -> Option<String> {
        self.name.take()
    }

    pub fn take_type(&mut self) -> Option<TypeId> {
        Some(TypeId::of::<I>())
    }

    pub fn take_help(&mut self) -> Option<Help> {
        self.help.take()
    }

    pub fn take_alias(&mut self) -> Option<Vec<String>> {
        self.alias.take()
    }

    pub fn take_styles(&mut self) -> Option<Vec<Style>> {
        self.styles.take()
    }

    pub fn take_action(&mut self) -> Option<Action> {
        self.action.take()
    }

    pub fn take_storer(&mut self) -> Option<ValStorer> {
        self.storer.take()
    }

    pub fn take_initializer(&mut self) -> Option<ValInitializer> {
        self.initializer.take()
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
        let mut output = parser.parse_opt(self).map_err(|e| e.into())?;
        let mut ret = <Self::Config>::default();

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
            ret.set_help(Help::default().with_help(v));
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

impl<I> ConfigBuild for ConfigBuilder<I> {
    type Infer = I;

    type Config = OptConfig;

    fn describe(&self) -> &str {
        todo!()
    }

    fn build<Parser>(mut self, parser: &Parser) -> Result<Self::Config, Error>
    where
        Parser: OptParser,
        Parser::Output: Information,
    {
        let mut ret = self
            .take_creator()
            .map(|v| v.build(parser))
            .unwrap_or(Ok(<Self::Config>::default()))?;

        if let Some(ctor) = self.take_ctor() {
            ret.set_ctor(ctor);
        }
        if let Some(r#type) = self.take_type() {
            ret.set_type_id(r#type);
        }
        if let Some(name) = self.take_name() {
            ret.set_name(name);
        }
        if let Some(force) = self.force {
            ret.set_force(force);
        }
        if let Some(index) = self.take_index() {
            ret.set_index(index);
        }
        if let Some(alias) = self.take_alias() {
            ret.set_alias(alias);
        }
        if let Some(help) = self.take_help() {
            ret.set_help(help);
        }
        if let Some(action) = self.take_action() {
            ret.set_action(action);
        }
        if let Some(storer) = self.take_storer() {
            ret.set_storer(storer);
        }
        if let Some(initializer) = self.take_initializer() {
            ret.set_initializer(initializer);
        }
        if let Some(ignore_name) = self.ignore_name {
            ret.set_ignore_name(ignore_name);
        }
        if let Some(ignore_alias) = self.ignore_alias {
            ret.set_ignore_alias(ignore_alias);
        }
        if let Some(ignore_index) = self.ignore_index {
            ret.set_ignore_index(ignore_index);
        }
        if let Some(styles) = self.take_styles() {
            ret.set_styles(styles);
        }
        Ok(ret)
    }
}
