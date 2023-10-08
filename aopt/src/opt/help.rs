/// The help information of option.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Help {
    /// The option hint is used in `usage`.
    hint: String,

    /// The option description used in `help`.
    help: String,

    /// The value hint of current option.
    value: String,
}

impl Help {
    pub fn new(hint: String, help: String, value: String) -> Self {
        Self { hint, help, value }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = help.into();
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn hint(&self) -> &str {
        &self.hint
    }

    pub fn help(&self) -> &str {
        &self.help
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_hint(&mut self, hint: impl Into<String>) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help(&mut self, help: impl Into<String>) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn set_value(&mut self, value: impl Into<String>) -> &mut Self {
        self.value = value.into();
        self
    }
}
