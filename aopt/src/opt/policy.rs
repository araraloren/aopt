#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ValPolicy {
    Set,

    App,

    Pop,

    Cnt,

    Bool,

    Null,
}

impl Default for ValPolicy {
    fn default() -> Self {
        ValPolicy::Null
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ValType {
    Bool,

    Int,

    Uint,

    Flt,

    Str,

    Null,
}

impl Default for ValType {
    fn default() -> Self {
        ValType::Null
    }
}
