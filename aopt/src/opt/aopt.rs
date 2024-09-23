use std::any::TypeId;

use crate::opt::Action;
#[allow(unused)]
use crate::opt::Cmd;
#[allow(unused)]
use crate::opt::Creator;
use crate::opt::Help;
use crate::opt::Index;
#[allow(unused)]
use crate::opt::Main;
use crate::opt::Opt;
#[allow(unused)]
use crate::opt::Pos;
use crate::opt::Style;
use crate::raise_error;
use crate::value::ErasedValue;
use crate::value::ValAccessor;
use crate::AStr;
use crate::Error;
use crate::Uid;

use super::ConfigValue;
use super::OptConfig;

/// A multiple features option type.
///
/// Some types support by default, they all implement [`Infer`](crate::value::Infer).
/// When create the option with `creator` using [`add_opt`](crate::set::OptSet::add_opt), the [`Creator`] will retrieve
/// other informations from the `infer type` list on the table.
/// When create the option from type using [`infer`](crate::opt::ConfigBuildInfer::infer), the [`Creator`] will retrieve
/// other informations from given type.
///
/// | type | action | ignore name | styles | index | alias | default value | creator | infer type |
/// |   -- |    --  |    -        |    --  |   -   |   -   |   --          |  -      |    --      |
/// | [`bool`] | [`Action::Set`] | `false` | [`Style::Combined`] [`Style::Boolean`] | no   | true | false | `b` | [`bool`] |
/// | [`i32`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | None | None |
/// | [`i64`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | `i`  | [`i64`] |
/// | [`u32`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | None | None |
/// | [`u64`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | `u`  | [`u64`] |
/// | [`f32`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | None | None |
/// | [`f64`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | `f`  | [`f64`] |
/// | [`usize`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | None | None |
/// | [`isize`] | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | None | None |
/// | [`String`]| [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | `s`  | [`String`] |
/// | [`OsString`](std::ffi::OsString) | [`Action::App`] | `false` | [`Style::Argument`] | no | true | None | `r` | [`OsString`](std::ffi::OsString) |
/// | [`Cmd`] | [`Action::Set`] | `false` | [`Style::Cmd`] | [`Forward(1)`](Index::Forward)  | true  |false | `c` | [`Cmd`] |
/// | [`Pos`] | [`Action::App`] | `true` | [`Style::Pos`] | yes | false | None | `p` | [`Pos`] |
/// | [`Main`] | [`Action::Null`] | `true` | [`Style::Main`] | [`AnyWhere`](Index::AnyWhere) | false | None | `m` | [`Main`] |
/// | [`Stdin`](std::io::Stdin) | [`Action::Set`] | [`false`] | [`Style::Boolean`] | None | true | None | None | [`Stdin`](std::io::Stdin) |
/// | [`Stop`](crate::value::Stop) | [`Action::Set`] | [`false`] | [`Style::Boolean`] | None |  true | None | None | [`Stop`](crate::value::Stop) |
///
/// For the value parser support, see [`RawValParser`](crate::value::RawValParser).
#[derive(Debug)]
pub struct AOpt {
    uid: Uid,

    name: AStr,

    r#type: TypeId,

    help: Help,

    styles: Vec<Style>,

    index: Option<Index>,

    accessor: ValAccessor,

    alias: Option<Vec<AStr>>,

    action: Action,

    matched: bool,

    force: bool,

    ignore_name: bool,

    ignore_alias: bool,

    ignore_index: bool,
}

impl AOpt {
    pub fn new(name: AStr, type_id: TypeId, accessor: ValAccessor) -> Self {
        Self {
            uid: 0,
            name,
            r#type: type_id,
            help: Default::default(),
            matched: false,
            force: false,
            action: Default::default(),
            styles: vec![],
            index: None,
            accessor,
            alias: None,
            ignore_name: false,
            ignore_alias: false,
            ignore_index: false,
        }
    }

    /// Set the unique identifier of option.
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    /// Set the name of option.
    pub fn with_name(mut self, name: AStr) -> Self {
        self.name = name;
        self
    }

    /// Set the type of option, see [`Ctor`](crate::set::Ctor).
    pub fn with_type(mut self, r#type: TypeId) -> Self {
        self.r#type = r#type;
        self
    }

    /// If the option will matching the name.
    pub fn with_ignore_name(mut self, ignore_name: bool) -> Self {
        self.ignore_name = ignore_name;
        self
    }

    /// If the option will matching the alias.
    pub fn with_ignore_alias(mut self, ignore_alias: bool) -> Self {
        self.ignore_alias = ignore_alias;
        self
    }

    /// If the option will matching the alias.
    pub fn with_ignore_index(mut self, ignore_index: bool) -> Self {
        self.ignore_index = ignore_index;
        self
    }

    /// Set the hint of option, such as `--option`.
    pub fn with_hint(mut self, hint: AStr) -> Self {
        self.help.set_hint(hint);
        self
    }

    /// Set the help message of option.
    pub fn with_help(mut self, help: AStr) -> Self {
        self.help.set_help(help);
        self
    }

    /// Set the value action of option.
    pub fn with_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    /// Set the help of option.
    pub fn with_opt_help(mut self, help: Help) -> Self {
        self.help = help;
        self
    }

    /// Set the [`Style`] of option.
    pub fn with_style(mut self, styles: Vec<Style>) -> Self {
        self.styles = styles;
        self
    }

    /// Set the NOA index of option.
    pub fn with_idx(mut self, index: Option<Index>) -> Self {
        self.index = index;
        self
    }

    /// If the option is force required.
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Set the alias of option.
    pub fn with_alias(mut self, alias: Option<Vec<AStr>>) -> Self {
        self.alias = alias;
        self
    }

    /// Set the value accessor of option, it will used by [`Policy`](crate::parser::Policy);
    pub fn with_accessor(mut self, value: ValAccessor) -> Self {
        self.accessor = value;
        self
    }
}

impl AOpt {
    pub fn set_name(&mut self, name: AStr) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_type(&mut self, r#type: TypeId) -> &mut Self {
        self.r#type = r#type;
        self
    }

    pub fn set_value(&mut self, value: ValAccessor) -> &mut Self {
        self.accessor = value;
        self
    }

    pub fn set_hint(&mut self, hint: AStr) -> &mut Self {
        self.help.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: AStr) -> &mut Self {
        self.help.set_help(help);
        self
    }

    pub fn set_action(&mut self, action: Action) -> &mut Self {
        self.action = action;
        self
    }

    pub fn set_style(&mut self, styles: Vec<Style>) -> &mut Self {
        self.styles = styles;
        self
    }

    pub fn set_index(&mut self, index: Option<Index>) -> &mut Self {
        self.index = index;
        self
    }

    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = force;
        self
    }

    pub fn add_alias(&mut self, name: AStr) -> &mut Self {
        if let Some(alias) = &mut self.alias {
            alias.push(name);
        }
        self
    }

    pub fn rem_alias(&mut self, name: &AStr) -> &mut Self {
        if let Some(alias) = &mut self.alias {
            if let Some((i, _)) = alias.iter().enumerate().find(|(_, v)| v == &name) {
                alias.remove(i);
            }
        }
        self
    }
}

impl Opt for AOpt {
    fn reset(&mut self) {
        self.set_matched(false);
    }

    fn uid(&self) -> Uid {
        self.uid
    }

    fn name(&self) -> &AStr {
        &self.name
    }

    fn r#type(&self) -> &TypeId {
        &self.r#type
    }

    fn hint(&self) -> &AStr {
        self.help.hint()
    }

    fn help(&self) -> &AStr {
        self.help.help()
    }

    fn valid(&self) -> bool {
        !self.force() || self.matched()
    }

    fn matched(&self) -> bool {
        self.matched
    }

    fn force(&self) -> bool {
        self.force
    }

    fn action(&self) -> &Action {
        &self.action
    }

    fn index(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn alias(&self) -> Option<&Vec<AStr>> {
        self.alias.as_ref()
    }

    fn accessor(&self) -> &ValAccessor {
        &self.accessor
    }

    fn accessor_mut(&mut self) -> &mut ValAccessor {
        &mut self.accessor
    }

    fn ignore_alias(&self) -> bool {
        self.ignore_alias
    }

    fn ignore_name(&self) -> bool {
        self.ignore_name
    }

    fn ignore_index(&self) -> bool {
        self.ignore_index
    }

    fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }

    fn set_matched(&mut self, matched: bool) {
        self.matched = matched;
    }

    fn mat_style(&self, style: Style) -> bool {
        self.styles.iter().any(|v| v == &style)
    }

    fn mat_force(&self, force: bool) -> bool {
        self.force() == force
    }

    fn mat_name(&self, name: Option<&str>) -> bool {
        name.iter().all(|&v| v == self.name().as_str())
    }

    fn mat_alias(&self, name: &str) -> bool {
        if let Some(alias) = &self.alias {
            alias.iter().any(|v| v == name)
        } else {
            false
        }
    }

    fn mat_index(&self, index: Option<(usize, usize)>) -> bool {
        if let Some((index, total)) = index {
            if let Some(realindex) = self.index() {
                if let Some(realindex) = realindex.calc_index(index, total) {
                    return realindex == index;
                }
            }
        }
        false
    }

    fn init(&mut self) -> Result<(), Error> {
        self.accessor.initialize()
    }
}

fn gen_hint(hint: Option<&AStr>, n: &AStr, idx: Option<&Index>, alias: Option<&Vec<AStr>>) -> AStr {
    let hint_generator = || {
        let mut names = Vec::with_capacity(1 + alias.map(|v| v.len()).unwrap_or_default());

        // add name
        names.push(n.as_str());
        // add alias
        if let Some(alias_vec) = alias {
            for alias in alias_vec {
                names.push(alias.as_str());
            }
        }
        // sort name by len
        names.sort_by_key(|v| v.len());
        crate::astr(if let Some(index) = idx {
            let index_string = index.to_help();

            // add index string
            if index_string.is_empty() {
                names.join(", ")
            } else {
                format!("{}@{}", names.join(", "), index_string)
            }
        } else {
            names.join(", ")
        })
    };

    hint.cloned().unwrap_or_else(hint_generator)
}

impl TryFrom<OptConfig> for AOpt {
    type Error = Error;

    fn try_from(mut value: OptConfig) -> Result<Self, Self::Error> {
        let r#type = value.take_type();
        let name = value.take_name();
        let force = value.take_force();
        let index = value.take_index();
        let alias = value.take_alias();
        let hint = value.take_hint();
        let help = value.take_help();
        let action = value.take_action();
        let storer = value.take_storer();
        let styles = value.take_style();
        let initializer = value.take_initializer();
        let ignore_name = value.ignore_name();
        let ignore_alias = value.ignore_alias();
        let ignore_index = value.ignore_index();

        let force = force.unwrap_or(false);
        let action = action.unwrap_or(Action::App);
        let storer = storer
            .ok_or_else(|| raise_error!("Incomplete option configuration: missing ValStorer"))?;
        let initializer = initializer.ok_or_else(|| {
            raise_error!("Incomplete option configuration: missing ValInitializer")
        })?;
        let styles =
            styles.ok_or_else(|| raise_error!("Incomplete option configuration: missing Style"))?;
        let name = name
            .ok_or_else(|| raise_error!("Incomplete option configuration: missing option name"))?;
        let hint = gen_hint(hint.as_ref(), &name, index.as_ref(), alias.as_ref());
        let help = help.unwrap_or_default();
        let r#type = r#type.ok_or_else(|| {
            raise_error!("Incomplete option configuration: missing option value type")
        })?;
        let help = Help::default().with_help(help).with_hint(hint);

        if ignore_alias {
            if let Some(alias) = &alias {
                debug_assert!(
                    !alias.is_empty(),
                    "Option {} not support alias: {:?}",
                    name,
                    alias
                );
            }
        }
        if ignore_index {
            if let Some(index) = &index {
                debug_assert!(
                    !index.is_null(),
                    "Please remove the index, option `{}` not support positional parameters: {:?}",
                    name,
                    index
                );
            }
        } else {
            debug_assert!(
                    index.is_some(),
                    "Please provide an index, indicate the position you want to capture for option `{}`.",
                    name
                );
        }
        Ok(
            AOpt::new(name, r#type, ValAccessor::new(storer, initializer))
                .with_force(force)
                .with_idx(index)
                .with_action(action)
                .with_alias(alias)
                .with_style(styles)
                .with_opt_help(help)
                .with_ignore_name(ignore_name)
                .with_ignore_alias(ignore_alias)
                .with_ignore_index(ignore_index),
        )
    }
}
