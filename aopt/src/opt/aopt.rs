use crate::opt::Action;
use crate::opt::Assoc;
#[allow(unused)]
use crate::opt::Creator;
use crate::opt::Help;
use crate::opt::Index;
use crate::opt::Opt;
use crate::opt::Style;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::ser::Services;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

/// A multiple features option type.
///
/// The type support by default:
///
/// |  creator   | assoc  | default action |string | ignore name | styles |
/// |  ----  | ----  | -- | -- | -- | -- |
/// | [`bool`](Creator::bool)  | [`Assoc::Bool`] | [`Action::App`] | `b` | false | [`Style::Boolean`],[`Style::Combined`] |
/// | [`str`](Creator::str)  | [`Assoc::Str`] | [`Action::App`] | `s` | false | [`Style::Argument`] |
/// | [`flt`](Creator::flt)  | [`Assoc::Flt`] | [`Action::App`] | `f` | false | [`Style::Argument`] |
/// | [`int`](Creator::int)  | [`Assoc::Int`] | [`Action::App`] | `i` | false | [`Style::Argument`] |
/// | [`uint`](Creator::uint)  | [`Assoc::Uint`] | [`Action::App`] | `u` | false | [`Style::Argument`] |
/// | [`cmd`](Creator::cmd)  | [`Assoc::Noa`] | [`Action::Set`] | `c` | false | [`Style::Cmd`] |
/// | [`pos`](Creator::pos)  | [`Assoc::Noa`] | [`Action::App`] | `p` | true | [`Style::Pos`] |
/// | [`main`](Creator::main)  | [`Assoc::Null`] | [`Action::Set`] | `m` | true | [`Style::Main`] |
/// | [`any`](Creator::any)  | [`Assoc::Null`] | [`Action::Null`] | `a` | false | except [`Style::Reserve`] |
///
/// |  creator   | index support  | optional support | alias support | validator |
/// |  ----  | ----  | -- | -- | -- |
/// | [`bool`](Creator::bool)  | no | yes | yes | [`bool`](ValValidator::bool) |
/// | [`str`](Creator::str)  | no | yes | yes | [`str`](ValValidator::str) |
/// | [`flt`](Creator::flt)  | no | yes | yes |  [`f64`](ValValidator::f64) |
/// | [`int`](Creator::int)  | no | yes | yes |  [`i64`](ValValidator::i64) |
/// | [`uint`](Creator::uint)  | no | yes | yes |  [`u64`](ValValidator::u64) |
/// | [`cmd`](Creator::cmd)  | [`Forward(1)`](crate::opt::Index::Forward) | `false` | no | [`some`](ValValidator::some) |
/// | [`pos`](Creator::pos)  | yes | yes | no |  [`some`](ValValidator::some) |
/// | [`main`](Creator::main)  | [`AnyWhere`](crate::opt::Index::AnyWhere) | no | no | [`null`](ValValidator::null) |
/// | [`any`](Creator::any)  | yes | yes |  yes | [`null`](ValValidator::null) |
#[derive(Debug, Default)]
pub struct AOpt {
    uid: Uid,

    name: Str,

    r#type: Str,

    help: Help,

    setted: bool,

    force: bool,

    assoc: Assoc,

    action: Action,

    styles: Vec<Style>,

    ignore_name_mat: bool,

    index: Option<Index>,

    validator: ValValidator,

    initiator: ValInitiator,

    alias: Option<Vec<Str>>,
}

impl AOpt {
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
    pub fn with_type(mut self, r#type: Str) -> Self {
        self.r#type = r#type;
        self
    }

    /// If the option will matching the name.
    pub fn with_ignore_name(mut self) -> Self {
        self.ignore_name_mat = true;
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

    /// Set the associated type of option.
    pub fn with_assoc(mut self, assoc: Assoc) -> Self {
        self.assoc = assoc;
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

    /// Set the value initiator of option, it will called by [`Policy`](crate::parser::Policy)
    /// initialize the option value.
    pub fn with_initiator(mut self, initiator: ValInitiator) -> Self {
        self.initiator = initiator;
        self
    }

    /// Set the value validator of option.
    pub fn with_validator(mut self, validator: ValValidator) -> Self {
        self.validator = validator;
        self
    }
}

impl AOpt {
    pub fn set_name(&mut self, name: Str) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_type(&mut self, r#type: Str) -> &mut Self {
        self.r#type = r#type;
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

    pub fn set_assoc(&mut self, assoc: Assoc) -> &mut Self {
        self.assoc = assoc;
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

    pub fn set_initiator(&mut self, initiator: ValInitiator) -> &mut Self {
        self.initiator = initiator;
        self
    }

    pub fn set_validator(&mut self, validator: ValValidator) -> &mut Self {
        self.validator = validator;
        self
    }
}

impl Opt for AOpt {
    fn reset(&mut self) {
        self.set_setted(false);
    }

    fn uid(&self) -> Uid {
        self.uid
    }

    fn name(&self) -> &Str {
        &self.name
    }

    fn r#type(&self) -> Str {
        self.r#type.clone()
    }

    fn hint(&self) -> &Str {
        self.help.hint()
    }

    fn help(&self) -> &Str {
        self.help.help()
    }

    fn valid(&self) -> bool {
        !self.force() || self.setted()
    }

    fn setted(&self) -> bool {
        self.setted
    }

    fn force(&self) -> bool {
        self.force
    }

    fn assoc(&self) -> &Assoc {
        &self.assoc
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

    fn set_setted(&mut self, setted: bool) {
        self.setted = setted;
    }

    fn mat_style(&self, style: Style) -> bool {
        self.styles.iter().any(|v| v == &style)
    }

    fn mat_force(&self, force: bool) -> bool {
        self.force() == force
    }

    fn mat_name(&self, name: Option<&Str>) -> bool {
        if self.ignore_name_mat {
            true
        } else {
            name.iter().all(|&v| v == self.name())
        }
    }

    fn mat_alias(&self, name: &Str) -> bool {
        if let Some(alias) = &self.alias {
            alias.iter().any(|v| v == name)
        } else {
            false
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

    fn check_val(&mut self, value: Option<&RawVal>, index: (usize, usize)) -> Result<bool, Error> {
        let name = self.name().clone();

        self.validator.check(name.as_str(), value, index)
    }

    fn init(&mut self, ser: &mut Services) -> Result<(), Error> {
        self.initiator.do_initialize(self.uid, ser)
    }
}
