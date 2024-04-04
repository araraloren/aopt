use crate::AStr;

/// The help information of option.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Help {
    /// The option hint is used in `usage`.
    hint: AStr,

    /// The option description used in `help`.
    help: AStr,
}

impl Help {
    pub fn new(hint: AStr, help: AStr) -> Self {
        Self { hint, help }
    }

    pub fn with_hint(mut self, hint: impl Into<AStr>) -> Self {
        self.hint = hint.into();
        self
    }

    pub fn with_help(mut self, help: impl Into<AStr>) -> Self {
        self.help = help.into();
        self
    }

    pub fn hint(&self) -> &AStr {
        &self.hint
    }

    pub fn help(&self) -> &AStr {
        &self.help
    }

    pub fn set_hint(&mut self, hint: impl Into<AStr>) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help(&mut self, help: impl Into<AStr>) -> &mut Self {
        self.help = help.into();
        self
    }
}
