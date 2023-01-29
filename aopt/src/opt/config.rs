use std::any::TypeId;

use crate::err::Error;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::Help;
use crate::opt::Index;
use crate::opt::Information;
use crate::opt::OptParser;
use crate::typeid;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::Str;

use super::Style;

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

    /// The style support by current option.
    fn style(&self) -> Option<&Vec<Style>>;

    /// The creator name of option.
    fn ctor(&self) -> Option<&Str>;

    /// The name of option.
    fn name(&self) -> Option<&Str>;

    /// The type name of option.
    fn r#type(&self) -> Option<&TypeId>;

    /// The index configuration of option.
    fn idx(&self) -> Option<&Index>;

    /// The alias name and prefix of option.
    fn alias(&self) -> Option<&Vec<Str>>;

    /// If the option is force required.
    fn force(&self) -> Option<bool>;

    /// Value action of option.
    fn action(&self) -> Option<&Action>;

    /// Value validator for option.
    fn storer(&self) -> Option<&ValStorer>;

    fn initializer(&self) -> Option<&ValInitializer>;

    fn ignore_name(&self) -> bool;

    fn support_alias(&self) -> bool;

    fn positional(&self) -> bool;

    fn fix_infer(&self) -> bool;

    fn has_idx(&self) -> bool;

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

    fn set_support_alias(&mut self, support_alias: bool) -> &mut Self;

    fn set_postional(&mut self, positional: bool) -> &mut Self;

    fn set_fix_infer(&mut self, fix_infer: bool) -> &mut Self;

    fn set_idx(&mut self, index: Index) -> &mut Self;

    fn set_force(&mut self, force: bool) -> &mut Self;

    fn set_ctor<S: Into<Str>>(&mut self, ctor: S) -> &mut Self;

    fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self;

    fn set_hint<S: Into<Str>>(&mut self, hint: S) -> &mut Self;

    fn set_help<S: Into<Str>>(&mut self, help: S) -> &mut Self;

    fn set_style(&mut self, styles: Vec<Style>) -> &mut Self;

    fn clr_alias(&mut self) -> &mut Self;

    fn add_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn rem_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn set_type<T: ErasedTy>(&mut self) -> &mut Self;

    fn set_action(&mut self, action: Action) -> &mut Self;

    fn set_storer(&mut self, storer: ValStorer) -> &mut Self;

    fn set_initializer(&mut self, initializer: ValInitializer) -> &mut Self;
}

/// Contain the information used for create option instance.
#[derive(Debug, Default)]
pub struct OptConfig {
    ctor: Option<Str>,

    ty: Option<TypeId>,

    name: Option<Str>,

    force: Option<bool>,

    idx: Option<Index>,

    alias: Vec<Str>,

    help: Help,

    action: Option<Action>,

    storer: Option<ValStorer>,

    initializer: Option<ValInitializer>,

    ignore_name: bool,

    support_alias: bool,

    postional: bool,

    fix_infer: bool,

    styles: Option<Vec<Style>>,
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

    pub fn with_type<T: ErasedTy>(mut self) -> Self {
        self.ty = Some(typeid::<T>());
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

    pub fn with_action(mut self, action: Option<Action>) -> Self {
        self.action = action;
        self
    }

    pub fn with_storer(mut self, storer: Option<ValStorer>) -> Self {
        self.storer = storer;
        self
    }

    pub fn with_initializer(mut self, initializer: Option<ValInitializer>) -> Self {
        self.initializer = initializer;
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
            .ok_or_else(|| Error::raise_error("Incomplete option configuration: missing Name"))?
            .clone())
    }

    pub fn gen_type(&mut self) -> Result<TypeId, Error> {
        self.ty
            .take()
            .ok_or_else(|| Error::raise_error("Incomplete option configuration: missing Type"))
    }

    pub fn gen_storer(&mut self) -> Result<ValStorer, Error> {
        self.storer
            .take()
            .ok_or_else(|| Error::raise_error("Incomplete option configuration: missing ValStorer"))
    }

    pub fn gen_initializer(&mut self) -> Result<ValInitializer, Error> {
        self.initializer.take().ok_or_else(|| {
            Error::raise_error("Incomplete option configuration: missing ValInitializer")
        })
    }

    pub fn gen_styles(&mut self) -> Result<Vec<Style>, Error> {
        self.styles
            .take()
            .ok_or_else(|| Error::raise_error("Incomplete option configuration: missing Style"))
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
        if let Some(v) = output.take_idx() {
            ret.set_idx(v);
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

    fn support_alias(&self) -> bool {
        self.support_alias
    }

    fn positional(&self) -> bool {
        self.postional
    }

    fn fix_infer(&self) -> bool {
        self.fix_infer
    }

    fn has_idx(&self) -> bool {
        self.idx.is_some()
    }

    fn has_ctor(&self) -> bool {
        self.ctor.is_some()
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

    fn set_support_alias(&mut self, ignore_alias: bool) -> &mut Self {
        self.support_alias = ignore_alias;
        self
    }

    fn set_postional(&mut self, ignore_index: bool) -> &mut Self {
        self.postional = ignore_index;
        self
    }

    fn set_fix_infer(&mut self, fix_infer: bool) -> &mut Self {
        self.fix_infer = fix_infer;
        self
    }

    fn set_idx(&mut self, index: Index) -> &mut Self {
        self.idx = Some(index);
        self
    }

    fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = Some(force);
        self
    }

    fn set_ctor<S: Into<Str>>(&mut self, ctor: S) -> &mut Self {
        self.ctor = Some(ctor.into());
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

    fn set_style(&mut self, styles: Vec<Style>) -> &mut Self {
        self.styles = Some(styles);
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

    fn set_type<T: ErasedTy>(&mut self) -> &mut Self {
        self.ty = Some(typeid::<T>());
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

pub(crate) fn fill_cfg<U, C>(info: &mut C)
where
    U: Infer,
    U::Val: RawValParser,
    C: ConfigValue + Default,
{
    let act = U::infer_act();
    let style = U::infer_style();
    let index = U::infer_index();
    let ignore_name = U::infer_ignore_name();
    let support_alias = U::infer_support_alias();
    let positional = U::infer_positional();
    let force = U::infer_force();
    let ctor = U::infer_ctor();
    let initializer = U::infer_initializer();
    let storer = if let Some(validator) = U::infer_validator() {
        Some(ValStorer::from(validator))
    } else {
        None
    };

    (!info.has_ctor()).then(|| info.set_ctor(ctor));
    (!info.has_idx()).then(|| index.map(|idx| info.set_idx(idx)));
    (!info.has_type()).then(|| info.set_type::<U::Val>());
    (!info.has_action()).then(|| info.set_action(act));
    (!info.has_style()).then(|| info.set_style(style));
    (!info.has_force()).then(|| info.set_force(force));
    (!info.has_action()).then(|| info.set_action(act));
    if let Some(storer) = storer {
        (!info.has_storer()).then(|| info.set_storer(storer));
    }
    if let Some(initializer) = initializer {
        (!info.has_initializer()).then(|| info.set_initializer(initializer));
    }
    info.set_ignore_name(ignore_name);
    info.set_support_alias(support_alias);
    info.set_postional(positional);
}

pub(crate) fn fill_cfg_infered<U, C>(info: &mut C)
where
    U: Infer,
    U::Val: RawValParser,
    C: ConfigValue + Default,
{
    let act = U::infer_act();
    let style = U::infer_style();
    let index = U::infer_index();
    let ignore_name = U::infer_ignore_name();
    let support_alias = U::infer_support_alias();
    let positional = U::infer_positional();
    let force = U::infer_force();
    let ctor = U::infer_ctor();
    let initializer = U::infer_initializer();
    let storer = if let Some(validator) = U::infer_validator() {
        Some(ValStorer::from(validator))
    } else {
        None
    };

    (!info.has_ctor()).then(|| info.set_ctor(ctor));
    (!info.has_idx()).then(|| index.map(|idx| info.set_idx(idx)));
    (!info.has_type()).then(|| info.set_type::<U::Val>());
    (!info.has_action()).then(|| info.set_action(act));
    (!info.has_style()).then(|| info.set_style(style));
    (!info.has_force()).then(|| info.set_force(force));
    (!info.has_action()).then(|| info.set_action(act));
    if info.fix_infer() {
        if let Some(storer) = storer {
            (!info.has_storer()).then(|| info.set_storer(storer));
        }
        if let Some(initializer) = initializer {
            (!info.has_initializer()).then(|| info.set_initializer(initializer));
        }
    }
    info.set_ignore_name(ignore_name);
    info.set_support_alias(support_alias);
    info.set_postional(positional);
}
