/// Option style
///
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Style {
    Null,

    /// The style indicate the option are set base on position(base on 1).
    Pos,

    /// The style indicate the option are set in first position.
    Cmd,

    /// The Main style option no need set, its callback will always be called.
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
