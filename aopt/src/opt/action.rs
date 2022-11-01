#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ValAction {
    Set,

    App,

    Pop,

    Cnt,

    Null,
}

impl Default for ValAction {
    fn default() -> Self {
        ValAction::Null
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ValAssoc {
    Bool,

    Int,

    Uint,

    Flt,

    Str,

    Null,
}

impl Default for ValAssoc {
    fn default() -> Self {
        ValAssoc::Null
    }
}
