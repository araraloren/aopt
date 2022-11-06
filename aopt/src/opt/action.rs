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

impl std::fmt::Display for ValAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValAction::Set => {
                write!(f, "ValAction::Set")
            }
            ValAction::App => {
                write!(f, "ValAction::App")
            }
            ValAction::Pop => {
                write!(f, "ValAction::Pop")
            }
            ValAction::Cnt => {
                write!(f, "ValAction::Cnt")
            }
            ValAction::Null => {
                write!(f, "ValAction::Null")
            }
        }
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

    Noa,

    Null,
}

impl Default for ValAssoc {
    fn default() -> Self {
        ValAssoc::Null
    }
}

impl std::fmt::Display for ValAssoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValAssoc::Bool => {
                write!(f, "ValAssoc::Bool")
            }
            ValAssoc::Int => {
                write!(f, "ValAssoc::Int")
            }
            ValAssoc::Uint => {
                write!(f, "ValAssoc::Uint")
            }
            ValAssoc::Flt => {
                write!(f, "ValAssoc::Flt")
            }
            ValAssoc::Str => {
                write!(f, "ValAssoc::Str")
            }
            ValAssoc::Noa => {
                write!(f, "ValAssoc::Noa")
            }
            ValAssoc::Null => {
                write!(f, "ValAssoc::Null")
            }
        }
    }
}
