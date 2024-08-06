use std::fmt::Display;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::ops::Deref;
use std::thread::AccessError;

use crate::RawVal;
use crate::Uid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Internal {
    UnexceptedPos,

    Failure(String),

    Error(String),

    MissingValue(String),

    PosRequired(Vec<String>),

    OptRequired(Vec<String>),

    CmdRequired(Vec<String>),

    OptionNotFound(String),

    ExtractValue(String),

    ThreadLocalAccess(String),

    RawValParse { val: String, hint: String },

    ArgsName { name: String, hint: String },

    Index { pat: String, hint: String },

    CreateStr { pat: String, hint: String },
}

impl Internal {
    pub fn display(&self) -> String {
        match self {
            Internal::RawValParse { val, hint } => {
                format!("invalid value `{val}`: {hint}",)
            }

            Internal::Failure(msg) => msg.clone(),

            Internal::Error(msg) => msg.clone(),

            Internal::ArgsName { name, hint } => {
                format!("invalid argument name `{name}`: {hint}")
            }

            Internal::UnexceptedPos => "can not insert Pos@1 if Cmd exist".to_owned(),

            Internal::Index { pat, hint } => {
                format!("invalid index string `{pat}`: {hint}")
            }
            Internal::CreateStr { pat, hint } => {
                format!("invalid option create string `{pat}`: {hint}")
            }
            Internal::MissingValue(name) => {
                format!("missing value for option `{name}`")
            }
            Internal::PosRequired(names) => match names.len() {
                1 => {
                    format!("positional `{}` is force required", names[0])
                }
                _ => {
                    format!("positional `{}` are force required", names.join(", "))
                }
            },
            Internal::OptRequired(names) => match names.len() {
                1 => {
                    format!("option `{}` is force required", names[0])
                }
                _ => {
                    format!("option `{}` are force required", names.join(", "))
                }
            },
            Internal::CmdRequired(names) => match names.len() {
                1 => {
                    format!("command `{}` is force required", names[0])
                }
                _ => {
                    format!("command `{}` are force required", names.join(", "))
                }
            },
            Internal::OptionNotFound(name) => {
                format!("unkown option `{name}`")
            }
            Internal::ExtractValue(msg) => {
                format!("extract value failed: `{msg}`")
            }
            Internal::ThreadLocalAccess(hint) => {
                format!("failed access thread local variable: {hint}")
            }
        }
    }
}

impl Display for Internal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::error::Error for Internal {}

#[derive(Debug, Clone)]
pub struct Error {
    uid: Option<Uid>,

    cause: Option<Box<Error>>,

    inner: Internal,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|v| v.deref() as &(dyn std::error::Error + 'static))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(uid) = self.uid {
            write!(f, "{} (uid = {})", self.display(), uid)
        } else {
            write!(f, "{}", self.display())
        }
    }
}

impl Error {
    pub fn new(error: Internal) -> Self {
        Self {
            inner: error,
            uid: None,
            cause: None,
        }
    }

    pub fn cause_by(mut self, cause_by: Self) -> Self {
        let _ = self.cause.insert(Box::new(cause_by));
        self
    }

    pub fn cause(self, error: Self) -> Self {
        error.cause_by(self)
    }

    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = Some(uid);
        self
    }

    pub fn uid(&self) -> Option<Uid> {
        self.uid
    }

    pub fn display(&self) -> String {
        self.inner.display()
    }

    /// The error can be moitted if [`is_failure`](Error::is_failure) return true.
    pub fn is_failure(&self) -> bool {
        if matches!(self.inner, Internal::RawValParse { val: _, hint: _ }) {
            true
        } else {
            matches!(
                self.inner,
                Internal::Failure(_)
                    | Internal::ExtractValue(_)
                    | Internal::OptionNotFound(_)
                    | Internal::CmdRequired(_)
                    | Internal::PosRequired(_)
                    | Internal::OptRequired(_)
                    | Internal::MissingValue(_)
            )
        }
    }

    pub fn from<E: std::error::Error + Display>(error: E) -> Self {
        Self::new(Internal::Error(format!("{}", error)))
    }

    pub fn args_name(name: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::new(Internal::ArgsName {
            name: name.into(),
            hint: hint.into(),
        })
    }

    pub fn sp_rawval(val: Option<&RawVal>, hint: impl Into<String>) -> Self {
        let val = if let Some(val) = val {
            format!("Some({})", val)
        } else {
            "None".to_string()
        };
        Self::new(Internal::RawValParse {
            val,
            hint: hint.into(),
        })
    }

    /// No Pos@1 allowed if the option set has cmd.
    pub fn unexcepted_pos() -> Self {
        Self::new(Internal::UnexceptedPos)
    }

    pub fn index(pat: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::new(Internal::Index {
            pat: pat.into(),
            hint: hint.into(),
        })
    }

    pub fn create_str(pat: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::new(Internal::CreateStr {
            pat: pat.into(),
            hint: hint.into(),
        })
    }

    pub fn local_access(hint: impl Into<String>) -> Self {
        Self::new(Internal::ThreadLocalAccess(hint.into()))
    }

    pub fn raise_error(hint: impl Into<String>) -> Self {
        Self::new(Internal::Error(hint.into()))
    }

    pub fn raise_failure(hint: impl Into<String>) -> Self {
        Self::new(Internal::Failure(hint.into()))
    }

    pub fn sp_missing_value(name: impl Into<String>) -> Self {
        Self::new(Internal::MissingValue(name.into()))
    }

    pub fn sp_pos_require<S: Into<String>>(names: Vec<S>) -> Self {
        Self::new(Internal::PosRequired(
            names.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn sp_opt_require<S: Into<String>>(names: Vec<S>) -> Self {
        Self::new(Internal::OptRequired(
            names.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn sp_cmd_require<S: Into<String>>(names: Vec<S>) -> Self {
        Self::new(Internal::CmdRequired(
            names.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn sp_not_found(name: impl Into<String>) -> Self {
        Self::new(Internal::OptionNotFound(name.into()))
    }

    pub fn sp_extract(msg: impl Into<String>) -> Self {
        Self::new(Internal::ExtractValue(msg.into()))
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Error::from(value)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::from(value)
    }
}

impl From<AccessError> for Error {
    fn from(value: AccessError) -> Self {
        Error::from(value)
    }
}

#[macro_export]
macro_rules! raise_error {
    ($($arg:tt)*) => {
        $crate::Error::raise_error(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! raise_failure {
    ($($arg:tt)*) => {
        $crate::Error::raise_failure(format!($($arg)*))
    };
}
