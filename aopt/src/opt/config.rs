use std::any::TypeId;

use crate::err::Error;
use crate::opt::Action;
use crate::opt::Index;
use crate::opt::Information;
use crate::opt::OptParser;
use crate::typeid;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::AStr;

use super::Style;

pub trait Config {
    fn new<Parser>(parser: &Parser, pattern: AStr) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information;
}

pub trait ConfigValue {
    /// The creator name of option.
    fn ctor(&self) -> Option<&AStr>;

    /// The [`TypeId`] of option.
    fn r#type(&self) -> Option<&TypeId>;

    /// The name of option.
    fn name(&self) -> Option<&AStr>;

    /// If the option is force required.
    fn force(&self) -> Option<bool>;

    /// The index configuration of option.
    fn index(&self) -> Option<&Index>;

    /// The alias name and prefix of option.
    fn alias(&self) -> Option<&Vec<AStr>>;

    /// The hint message used in usage of option.
    fn hint(&self) -> Option<&AStr>;

    /// The help message of option.
    fn help(&self) -> Option<&AStr>;

    /// Value action of option.
    fn action(&self) -> Option<&Action>;

    /// Value validator for option.
    fn storer(&self) -> Option<&ValStorer>;

    /// The style support by current option.
    fn style(&self) -> Option<&Vec<Style>>;

    /// Value initializer for option.
    fn initializer(&self) -> Option<&ValInitializer>;

    /// The creator name of option.
    fn ctor_mut(&mut self) -> Option<&mut AStr>;

    /// The [`TypeId`] of option.
    fn type_mut(&mut self) -> Option<&mut TypeId>;

    /// The name of option.
    fn name_mut(&mut self) -> Option<&mut AStr>;

    /// If the option is force required.
    fn force_mut(&mut self) -> Option<&mut bool>;

    /// The index configuration of option.
    fn index_mut(&mut self) -> Option<&mut Index>;

    /// The alias name and prefix of option.
    fn alias_mut(&mut self) -> Option<&mut Vec<AStr>>;

    /// The hint message used in usage of option.
    fn hint_mut(&mut self) -> Option<&mut AStr>;

    /// The help message of option.
    fn help_mut(&mut self) -> Option<&mut AStr>;

    /// Value action of option.
    fn action_mut(&mut self) -> Option<&mut Action>;

    /// Value validator for option.
    fn storer_mut(&mut self) -> Option<&mut ValStorer>;

    /// The style support by current option.
    fn style_mut(&mut self) -> Option<&mut Vec<Style>>;

    /// Value initializer for option.
    fn initializer_mut(&mut self) -> Option<&mut ValInitializer>;

    fn ignore_name(&self) -> bool;

    fn ignore_alias(&self) -> bool;

    fn ignore_index(&self) -> bool;

    fn has_ctor(&self) -> bool;

    fn has_type(&self) -> bool;

    fn has_name(&self) -> bool;

    fn has_force(&self) -> bool;

    fn has_index(&self) -> bool;

    fn has_hint(&self) -> bool;

    fn has_help(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_action(&self) -> bool;

    fn has_storer(&self) -> bool;

    fn has_style(&self) -> bool;

    fn has_initializer(&self) -> bool;

    fn set_ctor(&mut self, ctor: impl Into<AStr>) -> &mut Self;

    fn set_type<T: 'static>(&mut self) -> &mut Self;

    fn set_type_id(&mut self, type_id: TypeId) -> &mut Self;

    fn set_name(&mut self, name: impl Into<AStr>) -> &mut Self;

    fn set_force(&mut self, force: bool) -> &mut Self;

    fn set_index(&mut self, index: Index) -> &mut Self;

    fn set_alias(&mut self, alias: Vec<impl Into<AStr>>) -> &mut Self;

    fn clr_alias(&mut self) -> &mut Self;

    fn add_alias(&mut self, alias: impl Into<AStr>) -> &mut Self;

    fn rem_alias(&mut self, alias: impl Into<AStr>) -> &mut Self;

    fn set_hint(&mut self, hint: impl Into<AStr>) -> &mut Self;

    fn set_help(&mut self, help: impl Into<AStr>) -> &mut Self;

    fn set_action(&mut self, action: Action) -> &mut Self;

    fn set_storer(&mut self, storer: ValStorer) -> &mut Self;

    fn set_style(&mut self, styles: Vec<Style>) -> &mut Self;

    fn set_initializer(&mut self, initializer: ValInitializer) -> &mut Self;

    fn set_ignore_name(&mut self, ignore_name: bool) -> &mut Self;

    fn set_ignore_alias(&mut self, ignore_alias: bool) -> &mut Self;

    fn set_ignore_index(&mut self, ignore_index: bool) -> &mut Self;

    fn take_ctor(&mut self) -> Option<AStr>;

    fn take_type(&mut self) -> Option<TypeId>;

    fn take_name(&mut self) -> Option<AStr>;

    fn take_force(&mut self) -> Option<bool>;

    fn take_index(&mut self) -> Option<Index>;

    fn take_alias(&mut self) -> Option<Vec<AStr>>;

    fn take_hint(&mut self) -> Option<AStr>;

    fn take_help(&mut self) -> Option<AStr>;

    fn take_action(&mut self) -> Option<Action>;

    fn take_storer(&mut self) -> Option<ValStorer>;

    fn take_style(&mut self) -> Option<Vec<Style>>;

    fn take_initializer(&mut self) -> Option<ValInitializer>;
}

/// Contain the information used for create option instance.
#[derive(Debug, Default)]
pub struct OptConfig {
    ctor: Option<AStr>,

    r#type: Option<TypeId>,

    name: Option<AStr>,

    force: Option<bool>,

    index: Option<Index>,

    alias: Option<Vec<AStr>>,

    hint: Option<AStr>,

    help: Option<AStr>,

    action: Option<Action>,

    storer: Option<ValStorer>,

    initializer: Option<ValInitializer>,

    ignore_name: bool,

    ignore_alias: bool,

    ignore_index: bool,

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

    pub fn with_ctor(mut self, ctor: impl Into<AStr>) -> Self {
        self.ctor = Some(ctor.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<AStr>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_type<T: 'static>(mut self) -> Self {
        self.r#type = Some(typeid::<T>());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<AStr>) -> Self {
        self.help = Some(hint.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<AStr>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_alias(mut self, alias: Vec<impl Into<AStr>>) -> Self {
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
}

impl Config for OptConfig {
    fn new<Parser>(parser: &Parser, pattern: AStr) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser,
        Parser::Output: Information,
    {
        let mut output = parser.parse_opt(pattern).map_err(|e| e.into())?;
        let mut ret = Self::default();

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
            ret.set_alias(v);
        }
        Ok(ret)
    }
}

impl ConfigValue for OptConfig {
    fn ctor(&self) -> Option<&AStr> {
        self.ctor.as_ref()
    }

    fn r#type(&self) -> Option<&TypeId> {
        self.r#type.as_ref()
    }

    fn name(&self) -> Option<&AStr> {
        self.name.as_ref()
    }

    fn force(&self) -> Option<bool> {
        self.force
    }

    fn index(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn alias(&self) -> Option<&Vec<AStr>> {
        self.alias.as_ref()
    }

    fn hint(&self) -> Option<&AStr> {
        self.help.as_ref()
    }

    fn help(&self) -> Option<&AStr> {
        self.help.as_ref()
    }

    fn action(&self) -> Option<&Action> {
        self.action.as_ref()
    }

    fn storer(&self) -> Option<&ValStorer> {
        self.storer.as_ref()
    }

    fn style(&self) -> Option<&Vec<Style>> {
        self.styles.as_ref()
    }

    fn initializer(&self) -> Option<&ValInitializer> {
        self.initializer.as_ref()
    }

    fn ctor_mut(&mut self) -> Option<&mut AStr> {
        self.ctor.as_mut()
    }

    fn type_mut(&mut self) -> Option<&mut TypeId> {
        self.r#type.as_mut()
    }

    fn name_mut(&mut self) -> Option<&mut AStr> {
        self.name.as_mut()
    }

    fn force_mut(&mut self) -> Option<&mut bool> {
        self.force.as_mut()
    }

    fn index_mut(&mut self) -> Option<&mut Index> {
        self.index.as_mut()
    }

    fn alias_mut(&mut self) -> Option<&mut Vec<AStr>> {
        self.alias.as_mut()
    }

    fn hint_mut(&mut self) -> Option<&mut AStr> {
        self.hint.as_mut()
    }

    fn help_mut(&mut self) -> Option<&mut AStr> {
        self.help.as_mut()
    }

    fn action_mut(&mut self) -> Option<&mut Action> {
        self.action.as_mut()
    }

    fn storer_mut(&mut self) -> Option<&mut ValStorer> {
        self.storer.as_mut()
    }

    fn style_mut(&mut self) -> Option<&mut Vec<Style>> {
        self.styles.as_mut()
    }

    fn initializer_mut(&mut self) -> Option<&mut ValInitializer> {
        self.initializer.as_mut()
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

    fn has_ctor(&self) -> bool {
        self.ctor.is_some()
    }

    fn has_type(&self) -> bool {
        self.r#type.is_some()
    }

    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_force(&self) -> bool {
        self.force.is_some()
    }

    fn has_index(&self) -> bool {
        self.index.is_some()
    }

    fn has_hint(&self) -> bool {
        self.hint.is_some()
    }

    fn has_help(&self) -> bool {
        self.help.is_some()
    }

    fn has_alias(&self) -> bool {
        self.alias.is_some()
    }

    fn has_action(&self) -> bool {
        self.action.is_some()
    }

    fn has_storer(&self) -> bool {
        self.storer.is_some()
    }

    fn has_style(&self) -> bool {
        self.styles.is_some()
    }

    fn has_initializer(&self) -> bool {
        self.initializer.is_some()
    }

    fn set_ctor(&mut self, ctor: impl Into<AStr>) -> &mut Self {
        self.ctor = Some(ctor.into());
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

    fn set_name(&mut self, name: impl Into<AStr>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = Some(force);
        self
    }

    fn set_index(&mut self, index: Index) -> &mut Self {
        self.index = Some(index);
        self
    }

    fn set_alias(&mut self, alias: Vec<impl Into<AStr>>) -> &mut Self {
        self.alias = Some(alias.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }

    fn clr_alias(&mut self) -> &mut Self {
        if let Some(alias) = self.alias.as_mut() {
            alias.clear();
        }
        self
    }

    fn add_alias(&mut self, alias: impl Into<AStr>) -> &mut Self {
        self.alias.get_or_insert(vec![]).push(alias.into());
        self
    }

    fn rem_alias(&mut self, alias: impl Into<AStr>) -> &mut Self {
        let alias = alias.into();

        if let Some(v) = self.alias.as_mut() {
            for (index, value) in v.iter().enumerate() {
                if value == &alias {
                    v.remove(index);
                    break;
                }
            }
        }
        self
    }

    fn set_hint(&mut self, hint: impl Into<AStr>) -> &mut Self {
        self.hint = Some(hint.into());
        self
    }

    fn set_help(&mut self, help: impl Into<AStr>) -> &mut Self {
        self.help = Some(help.into());
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

    fn set_style(&mut self, styles: Vec<Style>) -> &mut Self {
        self.styles = Some(styles);
        self
    }

    fn set_initializer(&mut self, initializer: ValInitializer) -> &mut Self {
        self.initializer = Some(initializer);
        self
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

    fn take_ctor(&mut self) -> Option<AStr> {
        self.ctor.take()
    }

    fn take_type(&mut self) -> Option<TypeId> {
        self.r#type.take()
    }

    fn take_name(&mut self) -> Option<AStr> {
        self.name.take()
    }

    fn take_force(&mut self) -> Option<bool> {
        self.force.take()
    }

    fn take_index(&mut self) -> Option<Index> {
        self.index.take()
    }

    fn take_alias(&mut self) -> Option<Vec<AStr>> {
        self.alias.take()
    }

    fn take_hint(&mut self) -> Option<AStr> {
        self.hint.take()
    }

    fn take_help(&mut self) -> Option<AStr> {
        self.help.take()
    }

    fn take_action(&mut self) -> Option<Action> {
        self.action.take()
    }

    fn take_storer(&mut self) -> Option<ValStorer> {
        self.storer.take()
    }

    fn take_style(&mut self) -> Option<Vec<Style>> {
        self.styles.take()
    }

    fn take_initializer(&mut self) -> Option<ValInitializer> {
        self.initializer.take()
    }
}
