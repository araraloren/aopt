use std::mem::take;
use ustr::Ustr;

use super::Opt;
use crate::set::CreateInfo;

/// The help information of option.
///
/// The `hint` is suggestion of the option. In generally, it will
/// using in `usage` and `option list`.
///
/// The `help` is description of the option. In generally, it will
/// using in `option list`.
///
/// Such as we have a option hint `--link|-l` with help
/// `Set the link path`:
/// ```txt
/// usage: executable [--link|-l] **ARGS**
///
/// OPT:
/// --link|-l   Set the link path
/// ```
///
#[derive(Debug, Clone, Default)]
pub struct HelpInfo {
    hint: Ustr,
    help: Ustr,
}

impl HelpInfo {
    pub fn new(hint: Ustr, help: Ustr) -> Self {
        Self { hint, help }
    }

    pub fn get_hint(&self) -> Ustr {
        self.hint
    }

    pub fn get_help(&self) -> Ustr {
        self.help
    }

    pub fn get_hint_mut(&mut self) -> &mut Ustr {
        &mut self.hint
    }

    pub fn get_help_mut(&mut self) -> &mut Ustr {
        &mut self.help
    }

    pub fn set_hint<T: Into<Ustr>>(&mut self, hint: T) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<T: Into<Ustr>>(&mut self, help: T) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn clone_and_generate_hint(&self, opt: &dyn Opt) -> Self {
        Self {
            help: self.help,
            hint: format!(
                "{}{}{}={}{}",
                if opt.get_optional() { "[" } else { "<" },
                opt.get_prefix(),
                opt.get_name(),
                opt.get_type_name(),
                if opt.get_optional() { "]" } else { ">" },
            )
            .into(),
        }
    }
}

/// Convert the [`CreateInfo`] into [`HelpInfo`].
///
/// It will generate the help info if it not exist in [`CreateInfo`].
impl<'a> From<&'a mut CreateInfo> for HelpInfo {
    fn from(ci: &'a mut CreateInfo) -> Self {
        let mut help_info = take(ci.get_help_info_mut());
        Self {
            help: take(help_info.get_help_mut()),
            hint: if help_info.get_hint().is_empty() {
                create_help_hint(ci)
            } else {
                take(help_info.get_hint_mut())
            },
        }
    }
}

/// The help function using for generate help hint of option.
pub fn create_help_hint(ci: &CreateInfo) -> Ustr {
    let mut names = vec![String::default()];

    // adding prefix
    if let Some(prefix) = ci.get_prefix() {
        names[0] += prefix.as_ref();
    }
    // adding deactivate style
    if ci.get_support_deactivate_style() {
        names[0] += "/";
    }
    // adding name
    names[0] += ci.get_name().as_ref();

    // adding alias
    for alias in ci.get_alias() {
        if ci.get_support_deactivate_style() {
            for prefix in ci.get_support_prefix() {
                if alias.starts_with(prefix.as_str()) {
                    if let Some(name) = alias.get(prefix.len()..alias.len()) {
                        names.push(format!("{}/{}", prefix, name));
                        break;
                    }
                }
            }
        } else {
            names.push(format!("{}", alias));
        }
    }

    names.sort_by_key(|v1| v1.len());

    let mut ret = names.join(",");

    // adding index
    let index_string = ci.get_index().to_string();
    if !index_string.is_empty() {
        ret += &format!("@{}", index_string);
    }

    ret.into()
}
