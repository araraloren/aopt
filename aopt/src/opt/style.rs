/// Option style 
/// 
/// The [Parser](crate::parser::Parser) will generate [Context](crate::ctx::Context) which support
/// specified [`Style`]. 
/// And when [Matcher](crate::proc::Matcher) process the opt, it will check the 
/// [Opt](crate::opt::Opt) whether support that style.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Style {
    Boolean,

    Argument,

    Multiple,

    Pos,

    Cmd,

    Main,

    Other,

    Null,
}

impl Default for Style {
    fn default() -> Self {
        Self::Null
    }
}
