use crate::opt::Action;
use crate::opt::Assoc;
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
/// |  creator   | assoc  | default action |string | ignore name | styles | deactivate style |
/// |  ----  | ----  | -- | -- | -- | -- | -- |
/// | [`BoolCreator`](crate::opt::BoolCreator)  | [`Assoc::Bool`] | [`Action::App`] | `b` | false | [`Style::Boolean`],[`Style::Combined`] | yes |
/// | [`StrCreator`](crate::opt::StrCreator)  | [`Assoc::Str`] | [`Action::App`] | `s` | false | [`Style::Argument`] | no |
/// | [`FltCreator`](crate::opt::FltCreator)  | [`Assoc::Flt`] | [`Action::App`] | `f` | false | [`Style::Argument`] | no |
/// | [`IntCreator`](crate::opt::IntCreator)  | [`Assoc::Int`] | [`Action::App`] | `i` | false | [`Style::Argument`] | no |
/// | [`UintCreator`](crate::opt::UintCreator)  | [`Assoc::Uint`] | [`Action::App`] | `u` | false | [`Style::Argument`] | no |
/// | [`CmdCreator`](crate::opt::CmdCreator)  | [`Assoc::Noa`] | [`Action::Set`] | `c` | false | [`Style::Cmd`] | no |
/// | [`PosCreator`](crate::opt::PosCreator)  | [`Assoc::Noa`] | [`Action::App`] | `p` | true | [`Style::Pos`] | no |
/// | [`MainCreator`](crate::opt::MainCreator)  | [`Assoc::Null`] | [`Action::Set`] | `m` | true | [`Style::Main`] | no |
///
/// |  creator   | index support  | optional support | prefix support | alias support | validator |
/// |  ----  | ----  | -- | -- | -- | -- |
/// | [`BoolCreator`](crate::opt::BoolCreator)  | no | yes | yes | yes | [`bool`](ValValidator::bool) |
/// | [`StrCreator`](crate::opt::StrCreator)  | no | yes | yes | yes | [`str`](ValValidator::str) |
/// | [`FltCreator`](crate::opt::FltCreator)  | no | yes | yes | yes |  [`f64`](ValValidator::f64) |
/// | [`IntCreator`](crate::opt::IntCreator)  | no | yes | yes | yes |  [`i64`](ValValidator::i64) |
/// | [`UintCreator`](crate::opt::UintCreator)  | no | yes | yes | yes |  [`u64`](ValValidator::u64) |
/// | [`CmdCreator`](crate::opt::CmdCreator)  | [`Forward(1)`](crate::opt::Index::Forward) | `false` | no | no | [`some`](ValValidator::some) |
/// | [`PosCreator`](crate::opt::PosCreator)  | yes | yes | no | no |  [`some`](ValValidator::some) |
/// | [`MainCreator`](crate::opt::MainCreator)  | [`AnyWhere`](crate::opt::Index::AnyWhere) | no | no | no | [`null`](ValValidator::null) |
#[derive(Debug, Default)]
pub struct AOpt {
    uid: Uid,

    name: Str,

    r#type: Str,

    help: Help,

    prefix: Option<Str>,

    setted: bool,

    optional: bool,

    assoc: Assoc,

    action: Action,

    styles: Vec<Style>,

    ignore_name_mat: bool,

    deactivate_style: bool,

    index: Option<Index>,

    validator: ValValidator,

    initiator: ValInitiator,

    alias: Option<Vec<(Str, Str)>>,
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
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Set the prefix of option.
    pub fn with_prefix(mut self, prefix: Option<Str>) -> Self {
        self.prefix = prefix;
        self
    }

    /// Set the alias of option.
    pub fn with_alias(mut self, alias: Option<Vec<(Str, Str)>>) -> Self {
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

    /// If the option support deactivate style such as `--/bool`.
    pub fn with_deactivate_style(mut self, deactivate_style: bool) -> Self {
        self.deactivate_style = deactivate_style;
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

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn set_prefix(&mut self, prefix: Option<Str>) -> &mut Self {
        self.prefix = prefix;
        self
    }

    pub fn add_alias(&mut self, prefix: Str, name: Str) -> &mut Self {
        if let Some(alias) = &mut self.alias {
            alias.push((prefix, name));
        }
        self
    }

    pub fn rem_alias(&mut self, prefix: &Str, name: &Str) -> &mut Self {
        if let Some(alias) = &mut self.alias {
            if let Some((i, _)) = alias
                .iter()
                .enumerate()
                .find(|(_, v)| &v.0 == prefix && &v.1 == name)
            {
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

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.deactivate_style = deactivate_style;
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
        self.optional() || self.setted()
    }

    fn setted(&self) -> bool {
        self.setted
    }

    fn optional(&self) -> bool {
        self.optional
    }

    fn assoc(&self) -> &Assoc {
        &self.assoc
    }

    fn action(&self) -> &Action {
        &self.action
    }

    fn is_deactivate(&self) -> bool {
        self.deactivate_style
    }

    fn prefix(&self) -> Option<&Str> {
        self.prefix.as_ref()
    }

    fn idx(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn alias(&self) -> Option<&Vec<(Str, Str)>> {
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

    fn mat_optional(&self, optional: bool) -> bool {
        self.optional() == optional
    }

    fn mat_name(&self, name: Option<&Str>) -> bool {
        if self.ignore_name_mat {
            true
        } else {
            name.iter().all(|&v| v == self.name())
        }
    }

    fn mat_prefix(&self, prefix: Option<&Str>) -> bool {
        self.prefix() == prefix
    }

    fn mat_alias(&self, prefix: &Str, name: &Str) -> bool {
        if let Some(alias) = &self.alias {
            alias.iter().any(|v| &v.0 == prefix && &v.1 == name)
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

    fn check_val(
        &mut self,
        value: Option<&RawVal>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error> {
        let name = self.name().clone();

        self.validator.check(name.as_str(), value, disable, index)
    }

    fn init(&mut self, ser: &mut Services) -> Result<(), Error> {
        self.initiator.do_initialize(self.uid, ser)
    }
}
