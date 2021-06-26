
use crate::opt::Opt;

#[derive(Debug, Clone, Default)]
pub struct HelpInfo {
    hint: String,
    help: String,
}

impl HelpInfo {
    pub fn new(hint: String, help: String) -> Self {
        Self {
            hint, help,
        }
    }

    pub fn get_hint(&self) -> &str {
        self.hint.as_ref()
    }

    pub fn get_help(&self) -> &str {
        self.help.as_ref()
    }

    pub fn set_hint<T: Into<String>>(&mut self, hint: T) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<T: Into<String>>(&mut self, help: T) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn clone_only_hint(&self, opt: &dyn Opt) -> Self {
        Self {
            help: self.help.clone(),
            hint: format!(
                "{}{}{}={}{}",
                if opt.get_optional() { "[" } else { "<" },
                opt.get_prefix(),
                opt.get_name(),
                opt.get_type(),
                if opt.get_optional() { "]" } else { ">" },
            ),
        }
    }
}
