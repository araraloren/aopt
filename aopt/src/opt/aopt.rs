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
use crate::value::ErasedValHandler;
use crate::value::ValAccessor;
use crate::Error;
use crate::Str;
use crate::Uid;

/// A multiple features option type.
///
/// Some types support by default, see [`Infer`](crate::value::Infer):
///
/// | type | action | ignore name | styles | index support | default force required | alias | default value |
/// | --   |   --   |    --       |   --   |        --        |       --       |   --  |  -- |
/// | [`bool`] | [`Action::Set`] | [`false`] | [`Style::Combined`] [`Style::Boolean`] | no | false | true | false |
/// | [`i32`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`i64`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`u32`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`u64`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`f32`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`f64`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`usize`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`isize`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`String`] | [`Action::App`] | [`false`] | [`Style::Argument`] | no | false | true | None |
/// | [`Cmd`] | [`Action::Set`] | [`false`] | [`Style::Cmd`] | [`Forward(1)`](Index::Forward) | true | true | false |
/// | [`Pos`] | [`Action::App`] | [`true`] | [`Style::Pos`] | yes | false | false | None |
/// | [`Main`] | [`Action::Null`] | [`true`] | [`Style::Main`] | [`AnyWhere`](Index::AnyWhere) | false | false | None |
/// | [`Stdin`](std::io::Stdin) | [`Action::Set`] | [`true`] | [`Style::Pos`] | [`AnyWhere`](Index::AnyWhere) | false | true | None |
#[derive(Debug)]
pub struct AOpt {
    uid: Uid,

    name: Str,

    r#type: TypeId,

    help: Help,

    matched: bool,

    force: bool,

    action: Action,

    styles: Vec<Style>,

    ignore_name: bool,

    index: Option<Index>,

    accessor: ValAccessor,

    alias: Option<Vec<Str>>,
}

impl AOpt {
    pub fn new(name: Str, type_id: TypeId, accessor: ValAccessor) -> Self {
        Self {
            uid: 0,
            name,
            r#type: type_id,
            help: Default::default(),
            matched: false,
            force: false,
            action: Default::default(),
            styles: vec![],
            ignore_name: false,
            index: None,
            accessor,
            alias: None,
        }
    }

    /// Set the unique identifier of option.
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    /// Set the name of option.
    pub fn with_name(mut self, name: Str) -> Self {
        self.name = name;
        self
    }

    /// Set the type of option, see [`Ctor`](crate::set::Ctor).
    pub fn with_type(mut self, r#type: TypeId) -> Self {
        self.r#type = r#type;
        self
    }

    /// If the option will matching the name.
    pub fn with_ignore_name(mut self, value: bool) -> Self {
        self.ignore_name = value;
        self
    }

    /// Set the hint of option, such as `--option`.
    pub fn with_hint(mut self, hint: Str) -> Self {
        self.help.set_hint(hint);
        self
    }

    /// Set the help message of option.
    pub fn with_help(mut self, help: Str) -> Self {
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
    pub fn with_alias(mut self, alias: Option<Vec<Str>>) -> Self {
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
    pub fn set_name(&mut self, name: Str) -> &mut Self {
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

    pub fn set_hint(&mut self, hint: Str) -> &mut Self {
        self.help.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: Str) -> &mut Self {
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

    pub fn set_idx(&mut self, index: Option<Index>) -> &mut Self {
        self.index = index;
        self
    }

    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = force;
        self
    }

    pub fn add_alias(&mut self, name: Str) -> &mut Self {
        if let Some(alias) = &mut self.alias {
            alias.push(name);
        }
        self
    }

    pub fn rem_alias(&mut self, name: &Str) -> &mut Self {
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

    fn name(&self) -> &Str {
        &self.name
    }

    fn r#type(&self) -> &TypeId {
        &self.r#type
    }

    fn hint(&self) -> &Str {
        self.help.hint()
    }

    fn help(&self) -> &Str {
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

    fn idx(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn alias(&self) -> Option<&Vec<Str>> {
        self.alias.as_ref()
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

    fn mat_name(&self, name: Option<&Str>) -> bool {
        if self.ignore_name {
            true
        } else {
            name.iter().all(|&v| v == self.name())
        }
    }

    /// If don't have alias, then it's not support alias, will return true.
    fn mat_alias(&self, name: &Str) -> bool {
        if let Some(alias) = &self.alias {
            alias.iter().any(|v| v == name)
        } else {
            true
        }
    }

    fn mat_idx(&self, index: Option<(usize, usize)>) -> bool {
        if let Some((index, total)) = index {
            if let Some(realindex) = self.idx() {
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

    fn accessor(&self) -> &ValAccessor {
        &self.accessor
    }

    fn accessor_mut(&mut self) -> &mut ValAccessor {
        &mut self.accessor
    }
}
