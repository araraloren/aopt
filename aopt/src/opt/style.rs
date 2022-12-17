/// Option style
///
#[non_exhaustive]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Hash,
)]
pub enum Style {
    Null,

    /// The style indicate the `NOA` are set base on position(base on `1` !!!).
    Pos,

    /// The style indicate the `NOA` are set in first position(`@1`).
    Cmd,

    /// The Main style `NOA` no need set, its callback will always be called.
    Main,

    /// The style indicate option don't need argument, such as `--boolean`, `-b` or with no prefix `b`.
    Boolean,

    /// The style indicate the option need an argument, such as `--int=42`, `-i 42` or `--str=foo`.
    Argument,

    /// The style indicate option support set multiple option in one string, such as `-ade` means set `-a`, `-d` and `-e`.
    Combined,

    /// Reserve using for user define style option.
    Reserve(u64),
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
            Style::Reserve(val) => {
                write!(f, "Style::Reserve({val})")
            }
        }
    }
}
