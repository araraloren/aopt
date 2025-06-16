/// Option style
///
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Style {
    Null,

    /// The style indicate the `NOA` are set base on position.
    Pos,

    /// The style indicate the `NOA` are set in first position(`@1`).
    Cmd,

    /// The Main style `NOA` no need set, its callback will always be called.
    Main,

    /// The style indicate option don't need argument, such as `--boolean`, `-b` or with no prefix `b`.
    /// Using it with [`Boolean`](https://docs.rs/aopt/latest/aopt/parser/enum.UserStyle.html#variant.Boolean).
    Boolean,

    /// The style indicate the option need an argument, such as `--int=42`, `-i 42` or `--str=foo`.
    Argument,

    /// The style indicate option support set multiple option in one string, such as `-ade` means set `-a`, `-d` and `-e`.
    Combined,

    /// The style indicate option don't need argument, such as `--boolean`, `-b` or with no prefix `b`.
    /// Using it with [`Flag`](https://docs.rs/aopt/latest/aopt/parser/enum.UserStyle.html#variant.Flag).
    Flag,
}

impl Default for Style {
    fn default() -> Self {
        Self::Null
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::Null => {
                write!(f, "Style::Null")
            }
            Style::Pos => {
                write!(f, "Style::Pos")
            }
            Style::Cmd => {
                write!(f, "Style::Cmd")
            }
            Style::Main => {
                write!(f, "Style::Main")
            }
            Style::Boolean => {
                write!(f, "Style::Boolean")
            }
            Style::Argument => {
                write!(f, "Style::Argument")
            }
            Style::Combined => {
                write!(f, "Style::Combined")
            }
            Style::Flag => {
                write!(f, "Style::Flag")
            }
        }
    }
}
