use std::ops::Deref;

/// User set option style used for generate [`Process`](crate::proc::Process).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UserStyle {
    Main,

    /// NOA argument base on position.
    Pos,

    /// The first NOA argument.
    Cmd,

    /// Option set style like `--opt=value`, the value is set after `=`.
    EqualWithValue,

    /// Option set style like `--opt value`, the value is set in next argument.
    Argument,

    /// Option set style like `--i42`, the value set in the option string, only support one letter.
    EmbeddedValue,

    /// Option set style like `--opt42`, the value set in the option string, but suppport more than one letter.
    EmbeddedValuePlus,

    /// Option set style like `-abc`, thus set both boolean options `a`, `b` and `c`.
    CombinedOption,

    /// Option set style like `--bool`, only support boolean option.
    Boolean,

    /// Option set style like `--flag`, but the value will be set to None.
    Flag,
}

/// Manage the support option set style[`UserStyle`].
#[derive(Debug, Clone)]
pub struct OptStyleManager {
    styles: Vec<UserStyle>,
}

impl Default for OptStyleManager {
    fn default() -> Self {
        Self {
            styles: vec![
                UserStyle::EqualWithValue,
                UserStyle::Argument,
                UserStyle::Boolean,
                UserStyle::EmbeddedValue,
            ],
        }
    }
}

impl OptStyleManager {
    pub fn with(mut self, styles: Vec<UserStyle>) -> Self {
        self.styles = styles;
        self
    }

    pub fn set(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.styles = styles;
        self
    }

    pub fn remove(&mut self, style: UserStyle) -> &mut Self {
        if let Some((index, _)) = self.styles.iter().enumerate().find(|v| v.1 == &style) {
            self.styles.remove(index);
        }
        self
    }

    pub fn insert(&mut self, index: usize, style: UserStyle) -> &mut Self {
        self.styles.insert(index, style);
        self
    }

    pub fn push(&mut self, style: UserStyle) -> &mut Self {
        if !self.styles.iter().any(|v| v == &style) {
            self.styles.push(style);
        }
        self
    }
}

impl Deref for OptStyleManager {
    type Target = Vec<UserStyle>;

    fn deref(&self) -> &Self::Target {
        &self.styles
    }
}
