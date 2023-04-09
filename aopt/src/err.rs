use std::fmt::Display;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::ops::Deref;
use std::thread::AccessError;

use ahash::HashSet;

use crate::Uid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Internal {
    Null,
    OtherError(String),
    Error(String, bool),
    InvalidArgName(String),
    UnexceptedPos,
    InvalidOptIndex(String, String),
    InvalidCreateStr(String, String),
    MissingOptValue(String),
    PosForceRequired(String),
    OptForceRequired(String),
    CmdForceRequired(String),
    OptionNotFound(String),
    ExtractValueError(String),
    ThreadLocalAccess(String),
}

impl Default for Internal {
    fn default() -> Self {
        Self::Null
    }
}

impl Internal {
    pub fn display(&self) -> String {
        match self {
            Internal::Null => "Null".to_owned(),
            Internal::OtherError(error) => error.clone(),
            Internal::Error(msg, _) => msg.clone(),
            Internal::InvalidArgName(msg) => {
                format!("Invalid argument name: {}", msg)
            }
            Internal::UnexceptedPos => "Can not insert Pos@1 if Cmd exist".to_owned(),
            Internal::InvalidOptIndex(pat, msg) => {
                format!("Invalid index string `{}`: {}", pat, msg)
            }
            Internal::InvalidCreateStr(str, msg) => {
                format!("Invalid option create string `{}`: {}", str, msg)
            }
            Internal::MissingOptValue(hint) => {
                format!("Missing option value for `{}`", hint)
            }
            Internal::PosForceRequired(names) => {
                format!("Positional `{}` are force required", names)
            }
            Internal::OptForceRequired(names) => {
                format!("Option `{}` are force required", names)
            }
            Internal::CmdForceRequired(names) => {
                format!("Command `{}` are force required", names)
            }
            Internal::OptionNotFound(str) => {
                format!("Can not find option `{}`", str)
            }
            Internal::ExtractValueError(msg) => {
                format!("Extract value failed: {}", msg)
            }
            Internal::ThreadLocalAccess(msg) => {
                format!("Can not access thread local variable: `{}`", msg)
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

#[derive(Debug, Clone, Default)]
pub struct Error {
    inner: Internal,

    uids: HashSet<Uid>,

    cause: Option<Box<Error>>,
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
        write!(f, "{}", self.display())
    }
}

impl Error {
    pub fn new(error: Internal) -> Self {
        Self {
            inner: error,
            uids: HashSet::default(),
            cause: None,
        }
    }

    pub fn display(&self) -> String {
        self.inner.display()
    }

    pub fn null() -> Self {
        Self::new(Internal::Null)
    }

    pub fn is_null(&self) -> bool {
        matches!(self.inner, Internal::Null)
    }

    /// The error can be moitted if [`is_failure`](Error::is_failure) return true.
    pub fn is_failure(&self) -> bool {
        if let Internal::Error(_, fail) = &self.inner {
            *fail
        } else {
            matches!(
                self.inner,
                Internal::ExtractValueError(_)
                    | Internal::OptionNotFound(_)
                    | Internal::CmdForceRequired(_)
                    | Internal::PosForceRequired(_)
                    | Internal::OptForceRequired(_)
                    | Internal::MissingOptValue(_)
            )
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
        self.uids = {
            let mut set = HashSet::default();
            set.insert(uid);
            set
        };
        self
    }

    pub fn with_uids(mut self, uids: Vec<Uid>) -> Self {
        self.uids = HashSet::from_iter(uids.into_iter());
        self
    }

    pub fn uids(&self) -> &HashSet<Uid> {
        &self.uids
    }

    pub fn from<E: std::error::Error + Display>(error: E) -> Self {
        Self::new(Internal::OtherError(format!("{}", error)))
    }

    pub fn invalid_arg_name(msg: impl Into<String>) -> Self {
        Self::new(Internal::InvalidArgName(msg.into()))
    }

    /// No Pos@1 allowed if the option set has cmd.
    pub fn unexcepted_pos_if_has_cmd() -> Self {
        Self::new(Internal::UnexceptedPos)
    }

    pub fn invalid_opt_index(pat: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::new(Internal::InvalidOptIndex(pat.into(), msg.into()))
    }

    pub fn invalid_create_str(pat: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::new(Internal::InvalidCreateStr(pat.into(), msg.into()))
    }

    pub fn local_access(msg: impl Into<String>) -> Self {
        Self::new(Internal::ThreadLocalAccess(msg.into()))
    }

    pub fn raise_error(msg: impl Into<String>) -> Self {
        Self::new(Internal::Error(msg.into(), false))
    }

    pub fn raise_failure(msg: impl Into<String>) -> Self {
        Self::new(Internal::Error(msg.into(), true))
    }

    pub fn sp_missing_opt_value(names: impl Into<String>) -> Self {
        Self::new(Internal::MissingOptValue(names.into()))
    }

    pub fn sp_pos_force_require(names: impl Into<String>) -> Self {
        Self::new(Internal::PosForceRequired(names.into()))
    }

    pub fn sp_opt_force_require(names: impl Into<String>) -> Self {
        Self::new(Internal::OptForceRequired(names.into()))
    }

    pub fn sp_cmd_force_require(names: impl Into<String>) -> Self {
        Self::new(Internal::CmdForceRequired(names.into()))
    }

    pub fn sp_option_not_found(hint: impl Into<String>) -> Self {
        Self::new(Internal::OptionNotFound(hint.into()))
    }

    pub fn sp_raise_extract_error(msg: impl Into<String>) -> Self {
        Self::new(Internal::ExtractValueError(msg.into()))
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
