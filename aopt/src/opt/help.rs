use crate::Str;

/// The help information of option.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
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

    pub fn with_hint(mut self, hint: impl Into<Str>) -> Self {
        self.hint = hint.into();
        self
    }

    pub fn with_help(mut self, help: impl Into<Str>) -> Self {
        self.help = help.into();
        self
    }

    pub fn hint(&self) -> &Str {
        &self.hint
    }

    pub fn help(&self) -> &Str {
        &self.help
    }

    pub fn set_hint(&mut self, hint: impl Into<Str>) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help(&mut self, help: impl Into<Str>) -> &mut Self {
        self.help = help.into();
        self
    }
}
