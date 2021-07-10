use std::mem::take;

use super::Opt;
use crate::set::CreateInfo;

#[derive(Debug, Clone, Default)]
pub struct HelpInfo {
    hint: String,
    help: String,
}

impl HelpInfo {
    pub fn new(hint: String, help: String) -> Self {
        Self { hint, help }
    }

    pub fn get_hint(&self) -> &String {
        &self.hint
    }

    pub fn get_help(&self) -> &String {
        &self.help
    }

    pub fn get_hint_mut(&mut self) -> &mut String {
        &mut self.hint
    }

    pub fn get_help_mut(&mut self) -> &mut String {
        &mut self.help
    }

    pub fn set_hint<T: Into<String>>(&mut self, hint: T) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<T: Into<String>>(&mut self, help: T) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn clone_and_generate_hint(&self, opt: &dyn Opt) -> Self {
        Self {
            help: self.help.clone(),
            hint: format!(
                "{}{}{}={}{}",
                if opt.get_optional() { "[" } else { "<" },
                opt.get_prefix(),
                opt.get_name(),
                opt.get_type_name(),
                if opt.get_optional() { "]" } else { ">" },
            ),
        }
    }
}

/// Generate the help info if it not exist in [`CreateInfo`].
impl<'a> From<&'a mut CreateInfo> for HelpInfo {
    fn from(ci: &'a mut CreateInfo) -> Self {
        let mut help_info = take(ci.get_help_info_mut());
        let default_string = String::default();

        Self {
            help: take(help_info.get_help_mut()),
            hint: if help_info.get_hint().is_empty() {
                create_help_hint(
                    ci.get_optional(),
                    ci.get_prefix().as_ref().unwrap_or(&default_string).as_str(),
                    ci.get_name(),
                    ci.get_type_name(),
                )
            } else {
                take(help_info.get_hint_mut())
            },
        }
    }
}

pub fn create_help_hint(optional: bool, prefix: &str, name: &str, type_name: &str) -> String {
    format!(
        "{}{}{}={}{}",
        if optional { "[" } else { "<" },
        prefix,
        name,
        type_name,
        if optional { "]" } else { ">" },
    )
}
