use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::err::Error;
use crate::opt::Action;
use crate::opt::Index;
use crate::opt::Information;
use crate::opt::OptParser;
use crate::typeid;
use crate::value::Placeholder;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::AStr;

use super::BuiltInCtor;
use super::Style;

pub trait ConfigBuild<C> {
    type Val;

    fn build<P>(self, parser: &P) -> Result<C, Error>
    where
        P: OptParser,
        P::Output: Information;
}

impl<C: ConfigValue + Default> ConfigBuild<C> for &'_ str {
    type Val = Placeholder;

    fn build<P>(self, parser: &P) -> Result<C, Error>
    where
        P: OptParser,
        P::Output: Information,
    {
        let mut output = parser.parse_opt(self).map_err(Into::into)?;
        let mut ret = C::default();

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

impl<C: ConfigValue + Default> ConfigBuild<C> for AStr {
    type Val = Placeholder;

    fn build<P>(self, parser: &P) -> Result<C, Error>
    where
        P: OptParser,
        P::Output: Information,
    {
        <&str as ConfigBuild<C>>::build(self.as_str(), parser)
    }
}

impl<C: ConfigValue + Default> ConfigBuild<C> for String {
    type Val = Placeholder;

    fn build<P>(self, parser: &P) -> Result<C, Error>
    where
        P: OptParser,
        P::Output: Information,
    {
        <&str as ConfigBuild<C>>::build(self.as_str(), parser)
    }
}

impl<C: ConfigValue + Default> ConfigBuild<C> for &'_ String {
    type Val = Placeholder;

    fn build<P>(self, parser: &P) -> Result<C, Error>
    where
        P: OptParser,
        P::Output: Information,
    {
        <&str as ConfigBuild<C>>::build(self.as_str(), parser)
    }
}

impl<C: ConfigValue, I: 'static> ConfigBuild<C> for ConfigBuilder<C, I> {
    type Val = I;

    fn build<P>(self, _: &P) -> Result<C, Error>
    where
        P: OptParser,
        P::Output: Information,
    {
        Ok(self.config.with_type::<I>())
    }
}

impl<C, T, I> ConfigBuild<C> for ConfigBuilderWith<C, T, I>
where
    I: 'static,
    T: ConfigBuild<C>,
    C: ConfigValue + Default,
{
    type Val = I;

    fn build<P>(self, parser: &P) -> Result<C, Error>
    where
        P: OptParser,
        P::Output: Information,
    {
        let mut init = self.init.build(parser)?;
        let mut config = self.inner.build(parser)?;

        macro_rules! merge {
            ($has:ident, $set:ident, $take:ident) => {
                if !config.$has() {
                    if let Some(value) = init.$take() {
                        config.$set(value);
                    }
                }
            };
        }
        merge!(has_ctor, set_ctor, take_ctor);
        merge!(has_type, set_type_id, take_type);
        merge!(has_name, set_name, take_name);
        merge!(has_force, set_force, take_force);
        merge!(has_index, set_index, take_index);
        merge!(has_alias, set_alias, take_alias);
        merge!(has_hint, set_hint, take_hint);
        merge!(has_help, set_help, take_help);
        merge!(has_action, set_action, take_action);
        merge!(has_storer, set_storer, take_storer);
        merge!(has_style, set_style, take_style);
        merge!(has_initializer, set_initializer, take_initializer);
        config.set_ignore_name(config.ignore_name() || init.ignore_name());
        config.set_ignore_alias(config.ignore_alias() || init.ignore_alias());
        config.set_ignore_index(config.ignore_index() || init.ignore_index());
        Ok(config)
    }
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

    fn infer_builtin_ty(&mut self);

    fn with_index(self, index: Index) -> Self;

    fn with_force(self, force: bool) -> Self;

    fn with_ctor(self, ctor: impl Into<AStr>) -> Self;

    fn with_name(self, name: impl Into<AStr>) -> Self;

    fn with_type<T: 'static>(self) -> Self;

    fn with_hint(self, hint: impl Into<AStr>) -> Self;

    fn with_help(self, help: impl Into<AStr>) -> Self;

    fn with_alias(self, alias: Vec<impl Into<AStr>>) -> Self;

    fn with_style(self, styles: Vec<Style>) -> Self;

    fn with_action(self, action: Action) -> Self;

    fn with_storer(self, storer: ValStorer) -> Self;

    fn with_ignore_alias(self, ignore_alias: bool) -> Self;

    fn with_ignore_index(self, ignore_index: bool) -> Self;

    fn with_ignore_name(self, ignore_name: bool) -> Self;

    fn with_initializer(self, initializer: ValInitializer) -> Self;
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

    fn infer_builtin_ty(&mut self) {
        if let Some(ctor) = self.ctor() {
            let built_in_ctor = BuiltInCtor::from_name(ctor);

            self.set_type_id(match built_in_ctor {
                BuiltInCtor::Int => typeid::<i64>(),
                BuiltInCtor::AStr => typeid::<String>(),
                BuiltInCtor::Flt => typeid::<f64>(),
                BuiltInCtor::Uint => typeid::<u64>(),
                BuiltInCtor::Bool => typeid::<bool>(),
                BuiltInCtor::Cmd => typeid::<crate::opt::Cmd>(),
                BuiltInCtor::Pos => typeid::<crate::opt::Pos<bool>>(),
                BuiltInCtor::Main => typeid::<crate::opt::Main>(),
                BuiltInCtor::Any => typeid::<crate::opt::Any>(),
                BuiltInCtor::Raw => typeid::<std::ffi::OsString>(),
                BuiltInCtor::Fallback => {
                    unreachable!("Fallback creator can't infer any type")
                }
            });
        }
    }

    fn with_index(mut self, index: Index) -> Self {
        self.index = Some(index);
        self
    }

    fn with_force(mut self, force: bool) -> Self {
        self.force = Some(force);
        self
    }

    fn with_ctor(mut self, ctor: impl Into<AStr>) -> Self {
        self.ctor = Some(ctor.into());
        self
    }

    fn with_name(mut self, name: impl Into<AStr>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn with_type<T: 'static>(mut self) -> Self {
        self.r#type = Some(typeid::<T>());
        self
    }

    fn with_hint(mut self, hint: impl Into<AStr>) -> Self {
        self.help = Some(hint.into());
        self
    }

    fn with_help(mut self, help: impl Into<AStr>) -> Self {
        self.help = Some(help.into());
        self
    }

    fn with_alias(mut self, alias: Vec<impl Into<AStr>>) -> Self {
        self.alias = Some(alias.into_iter().map(|v| v.into()).collect());
        self
    }

    fn with_style(mut self, styles: Vec<Style>) -> Self {
        self.styles = Some(styles);
        self
    }

    fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    fn with_storer(mut self, storer: ValStorer) -> Self {
        self.storer = Some(storer);
        self
    }

    fn with_ignore_alias(mut self, ignore_alias: bool) -> Self {
        self.ignore_alias = ignore_alias;
        self
    }

    fn with_ignore_index(mut self, ignore_index: bool) -> Self {
        self.ignore_index = ignore_index;
        self
    }

    fn with_ignore_name(mut self, ignore_name: bool) -> Self {
        self.ignore_name = ignore_name;
        self
    }

    fn with_initializer(mut self, initializer: ValInitializer) -> Self {
        self.initializer = Some(initializer);
        self
    }
}

pub trait ConfigBuildMutable {
    type Cfg;

    fn config_mut(&mut self) -> &mut Self::Cfg;
}

impl<C, I> ConfigBuildMutable for ConfigBuilder<C, I> {
    type Cfg = C;

    fn config_mut(&mut self) -> &mut C {
        &mut self.config
    }
}

impl<C, T, I> ConfigBuildMutable for ConfigBuilderWith<C, T, I>
where
    C: Default,
{
    type Cfg = C;

    fn config_mut(&mut self) -> &mut C {
        self.inner_mut().config_mut()
    }
}

pub trait ConfigBuildInfer<C> {
    type Output<T>;

    fn infer<T: 'static>(self) -> Self::Output<T>;
}

impl<C> ConfigBuildInfer<C> for &'_ str
where
    C: ConfigValue + Default,
{
    type Output<T> = ConfigBuilderWith<C, Self, T>;

    fn infer<T: 'static>(self) -> Self::Output<T> {
        ConfigBuilderWith::new(self, ConfigBuilder::new(C::default().with_type::<T>()))
    }
}

impl<C> ConfigBuildInfer<C> for AStr
where
    C: ConfigValue + Default,
{
    type Output<T> = ConfigBuilderWith<C, Self, T>;

    fn infer<T: 'static>(self) -> Self::Output<T> {
        ConfigBuilderWith::new(self, ConfigBuilder::new(C::default().with_type::<T>()))
    }
}

impl<C> ConfigBuildInfer<C> for String
where
    C: ConfigValue + Default,
{
    type Output<T> = ConfigBuilderWith<C, Self, T>;

    fn infer<T: 'static>(self) -> Self::Output<T> {
        ConfigBuilderWith::new(self, ConfigBuilder::new(C::default().with_type::<T>()))
    }
}

impl<C> ConfigBuildInfer<C> for &'_ String
where
    C: ConfigValue + Default,
{
    type Output<T> = ConfigBuilderWith<C, Self, T>;

    fn infer<T: 'static>(self) -> Self::Output<T> {
        ConfigBuilderWith::new(self, ConfigBuilder::new(C::default().with_type::<T>()))
    }
}

impl<C, I> ConfigBuildInfer<C> for ConfigBuilder<C, I> {
    type Output<T> = ConfigBuilder<C, T>;

    fn infer<T: 'static>(self) -> Self::Output<T> {
        ConfigBuilder {
            config: self.config,
            marker: PhantomData,
        }
    }
}

pub trait ConfigBuildWith {
    type Output;

    fn with_ctor(self, ctor: impl Into<AStr>) -> Self::Output;

    fn with_name(self, name: impl Into<AStr>) -> Self::Output;

    fn with_force(self, force: bool) -> Self::Output;

    fn with_index(self, index: Index) -> Self::Output;

    fn with_alias(self, alias: Vec<impl Into<AStr>>) -> Self::Output;

    fn with_hint(self, hint: impl Into<AStr>) -> Self::Output;

    fn with_help(self, help: impl Into<AStr>) -> Self::Output;

    fn with_action(self, action: Action) -> Self::Output;

    fn with_storer(self, storer: ValStorer) -> Self::Output;

    fn with_initializer(self, initializer: ValInitializer) -> Self::Output;

    fn with_ignore_alias(self, ignore_alias: bool) -> Self::Output;

    fn with_ignore_index(self, ignore_index: bool) -> Self::Output;

    fn with_ignore_name(self, ignore_name: bool) -> Self::Output;

    fn with_style(self, styles: Vec<Style>) -> Self::Output;
}

impl<T> ConfigBuildWith for T
where
    T: ConfigBuildMutable,
    T::Cfg: ConfigValue,
{
    type Output = Self;

    fn with_ctor(mut self, ctor: impl Into<AStr>) -> Self::Output {
        self.config_mut().set_ctor(ctor);
        self
    }

    fn with_name(mut self, name: impl Into<AStr>) -> Self::Output {
        self.config_mut().set_name(name);
        self
    }

    fn with_force(mut self, force: bool) -> Self::Output {
        self.config_mut().set_force(force);
        self
    }

    fn with_index(mut self, index: Index) -> Self::Output {
        self.config_mut().set_index(index);
        self
    }

    fn with_alias(mut self, alias: Vec<impl Into<AStr>>) -> Self::Output {
        self.config_mut().set_alias(alias);
        self
    }

    fn with_hint(mut self, hint: impl Into<AStr>) -> Self::Output {
        self.config_mut().set_hint(hint);
        self
    }

    fn with_help(mut self, help: impl Into<AStr>) -> Self::Output {
        self.config_mut().set_help(help);
        self
    }

    fn with_action(mut self, action: Action) -> Self::Output {
        self.config_mut().set_action(action);
        self
    }

    fn with_storer(mut self, storer: ValStorer) -> Self::Output {
        self.config_mut().set_storer(storer);
        self
    }

    fn with_initializer(mut self, initializer: ValInitializer) -> Self::Output {
        self.config_mut().set_initializer(initializer);
        self
    }

    fn with_ignore_alias(mut self, ignore_alias: bool) -> Self::Output {
        self.config_mut().set_ignore_alias(ignore_alias);
        self
    }

    fn with_ignore_index(mut self, ignore_index: bool) -> Self::Output {
        self.config_mut().set_ignore_index(ignore_index);
        self
    }

    fn with_ignore_name(mut self, ignore_name: bool) -> Self::Output {
        self.config_mut().set_ignore_name(ignore_name);
        self
    }

    fn with_style(mut self, styles: Vec<Style>) -> Self::Output {
        self.config_mut().set_style(styles);
        self
    }
}

pub struct ConfigBuilder<C, I> {
    config: C,

    marker: PhantomData<I>,
}

impl<C, I> Default for ConfigBuilder<C, I>
where
    C: Default,
{
    fn default() -> Self {
        Self {
            config: C::default(),
            marker: Default::default(),
        }
    }
}

impl<C, I> Debug for ConfigBuilder<C, I>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigBuilder")
            .field("config", &self.config)
            .finish()
    }
}

impl<C, I> ConfigBuilder<C, I> {
    pub fn new(config: C) -> Self {
        Self {
            config,
            marker: PhantomData,
        }
    }

    pub fn config(&self) -> &C {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut C {
        &mut self.config
    }
}

pub struct ConfigBuilderWith<C, T, I> {
    init: T,
    inner: ConfigBuilder<C, I>,
}

impl<C, T, I> Default for ConfigBuilderWith<C, T, I>
where
    T: Default,
    C: Default,
{
    fn default() -> Self {
        Self {
            init: Default::default(),
            inner: Default::default(),
        }
    }
}

impl<C, T, I> Debug for ConfigBuilderWith<C, T, I>
where
    T: Debug,
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigBuilderWith")
            .field("init", &self.init)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<C, T, I> ConfigBuilderWith<C, T, I> {
    pub fn new(init: T, inner: ConfigBuilder<C, I>) -> Self {
        Self { init, inner }
    }

    pub fn inner(&self) -> &ConfigBuilder<C, I> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut ConfigBuilder<C, I> {
        &mut self.inner
    }

    pub fn config(&self) -> &C {
        self.inner.config()
    }

    pub fn config_mut(&mut self) -> &mut C {
        self.inner.config_mut()
    }
}

impl<C, T, I> From<T> for ConfigBuilderWith<C, T, I>
where
    C: Default,
{
    fn from(value: T) -> Self {
        ConfigBuilderWith::new(value, Default::default())
    }
}

macro_rules! def_help_for {
    ($type:ty) => {
        impl ConfigBuildWith for $type {
            type Output = ConfigBuilderWith<OptConfig, Self, Placeholder>;

            fn with_ctor(self, ctor: impl Into<AStr>) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_ctor(ctor)),
                )
            }

            fn with_name(self, name: impl Into<AStr>) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_name(name)),
                )
            }

            fn with_force(self, force: bool) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_force(force)),
                )
            }

            fn with_index(self, index: Index) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_index(index)),
                )
            }

            fn with_alias(self, alias: Vec<impl Into<AStr>>) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_alias(alias)),
                )
            }

            fn with_hint(self, hint: impl Into<AStr>) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_hint(hint)),
                )
            }

            fn with_help(self, help: impl Into<AStr>) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_help(help)),
                )
            }

            fn with_action(self, action: Action) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_action(action)),
                )
            }

            fn with_storer(self, storer: ValStorer) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_storer(storer)),
                )
            }

            fn with_initializer(self, initializer: ValInitializer) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_initializer(initializer)),
                )
            }

            fn with_ignore_alias(self, ignore_alias: bool) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_ignore_alias(ignore_alias)),
                )
            }

            fn with_ignore_index(self, ignore_index: bool) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_ignore_index(ignore_index)),
                )
            }

            fn with_ignore_name(self, ignore_name: bool) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_ignore_name(ignore_name)),
                )
            }

            fn with_style(self, styles: Vec<Style>) -> Self::Output {
                ConfigBuilderWith::new(
                    self,
                    ConfigBuilder::new(OptConfig::default().with_style(styles)),
                )
            }
        }
    };
}

def_help_for!(AStr);
def_help_for!(&'_ str);
def_help_for!(&'_ String);
def_help_for!(String);
