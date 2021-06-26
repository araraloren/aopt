
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Style {
    Boolean,

    Argument,

    Multiple,

    Pos,

    Cmd,

    Main,

    Null,
}

impl Default for Style {
    fn default() -> Self {
        Self::Null
    }
}