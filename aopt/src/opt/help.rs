use crate::Str;

/// The help information of option.
///
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Help {
    /// The option hint is used in `usage`.
    hint: Str,

    /// The option description used in `help`.
    help: Str,
}

impl Help {
    pub fn new(hint: Str, help: Str) -> Self {
        Self { hint, help }
    }

    pub fn with_hint<T: Into<Str>>(mut self, hint: T) -> Self {
        self.hint = hint.into();
        self
    }

    pub fn with_help<T: Into<Str>>(mut self, help: T) -> Self {
        self.help = help.into();
        self
    }

    pub fn hint(&self) -> &Str {
        &self.hint
    }

    pub fn help(&self) -> &Str {
        &self.help
    }

    pub fn set_hint<T: Into<Str>>(&mut self, hint: T) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<T: Into<Str>>(&mut self, help: T) -> &mut Self {
        self.help = help.into();
        self
    }
}
