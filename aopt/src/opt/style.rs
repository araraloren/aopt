/// Option style
///
/// The [`Parser`](crate::parser::Parser) will generate [`Context`](crate::ctx::Context) which support
/// specified [`Style`].
/// And when [`Matcher`](crate::proc::Matcher) process the opt, it will check the
/// [`Opt`](crate::opt::Opt) whether support that style.
///
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Style {
    /// The style indicate option don't need argument, such as `--boolean`, `-b` or with no prefix `b`.
    Boolean,

    /// The style indicate the option need an argument, such as `--int=42`, `-i 42` or `--str=foo`.
    Argument,

    /// The style indicate option support set multiple option in one string, such as `-ade` means set `-a`, `-d` and `-e`.
    Multiple,

    /// The style indicate the non-option is set base on position(base on 1).
    Pos,

    /// The style indicate the non-option is set in first position.
    Cmd,

    /// The Main style non-option no need set, its callback will always be called.
    Main,

    /// Reserve using for user define style option.
    Other,

    Null,
}

impl Default for Style {
    fn default() -> Self {
        Self::Null
    }
}
