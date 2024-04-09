use std::fmt::Display;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::ops::Deref;
use std::thread::AccessError;

use crate::Uid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum ErrorCmd {
    StopPolicy,
    QuitPolicy,
}

#[derive(Debug, Clone)]
pub enum Internal {
    Null,

    UnexceptedPos,

    Command(ErrorCmd),

    RawValParse(String),

    Failure(String),

    Error(String),

    ArgsName(String),

    Index(String),

    CreateStr(String),

    MissingValue(String),

    PosRequired(String),

    OptRequired(String),

    CmdRequired(String),

    OptionNotFound(String),

    ExtractValue(String),

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

            Internal::RawValParse(msg) => {
                format!("Failed parsing raw value: `{msg}`")
            }

            Internal::Command(command) => {
                format!("Command using for policy: {:?}", command)
            }

            Internal::Failure(msg) => msg.clone(),

            Internal::Error(msg) => msg.clone(),

            Internal::ArgsName(msg) => {
                format!("Invalid argument name: {msg}")
            }

            Internal::UnexceptedPos => "Can not insert Pos@1 if Cmd exist".to_owned(),

            Internal::Index(msg) => {
                format!("Invalid index string : {msg}")
            }
            Internal::CreateStr(msg) => {
                format!("Invalid option create string: {msg}")
            }
            Internal::MissingValue(hint) => {
                format!("Missing value for `{hint}`")
            }
            Internal::PosRequired(names) => {
                format!("Positional `{names}` are force required")
            }
            Internal::OptRequired(names) => {
                format!("Option `{names}` are force required",)
            }
            Internal::CmdRequired(names) => {
                format!("Command `{names}` are force required",)
            }
            Internal::OptionNotFound(name) => {
                format!("Can not find option `{name}`")
            }
            Internal::ExtractValue(msg) => {
                format!("Extract value failed: `{msg}`")
            }
            Internal::ThreadLocalAccess(msg) => {
                format!("Can not access thread local variable: `{msg}`")
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

    pub fn null() -> Self {
        Self::new(Internal::Null)
    }

    pub fn uid(&self) -> Option<Uid> {
        self.uid
    }

    pub fn display(&self) -> String {
        self.inner.display()
    }

    pub fn command(&self) -> Option<ErrorCmd> {
        if let Internal::Command(cmd) = self.inner {
            Some(cmd)
        } else {
            None
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self.inner, Internal::Null)
    }

    /// The error can be moitted if [`is_failure`](Error::is_failure) return true.
    pub fn is_failure(&self) -> bool {
        if matches!(self.inner, Internal::Command(_)) {
            true
        } else {
            matches!(
                self.inner,
                Internal::Failure(_)
                    | Internal::RawValParse(_)
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

    pub fn raise_args_name(msg: impl Into<String>) -> Self {
        Self::new(Internal::ArgsName(msg.into()))
    }

    pub fn raise_sp_rawval(msg: impl Into<String>) -> Self {
        Self::new(Internal::RawValParse(msg.into()))
    }

    /// No Pos@1 allowed if the option set has cmd.
    pub fn unexcepted_pos_if_has_cmd() -> Self {
        Self::new(Internal::UnexceptedPos)
    }

    pub fn raise_index(msg: impl Into<String>) -> Self {
        Self::new(Internal::Index(msg.into()))
    }

    pub fn raise_create_str(pat: impl Into<String>) -> Self {
        Self::new(Internal::CreateStr(pat.into()))
    }

    pub fn raise_local_access(msg: impl Into<String>) -> Self {
        Self::new(Internal::ThreadLocalAccess(msg.into()))
    }

    pub fn raise_error(msg: impl Into<String>) -> Self {
        Self::new(Internal::Error(msg.into()))
    }

    pub fn raise_failure(msg: impl Into<String>) -> Self {
        Self::new(Internal::Failure(msg.into()))
    }

    pub fn raise_command(cmd: ErrorCmd) -> Self {
        Self::new(Internal::Command(cmd))
    }

    pub fn raise_sp_missing_value(names: impl Into<String>) -> Self {
        Self::new(Internal::MissingValue(names.into()))
    }

    pub fn raise_sp_pos_require(names: impl Into<String>) -> Self {
        Self::new(Internal::PosRequired(names.into()))
    }

    pub fn raise_sp_opt_require(names: impl Into<String>) -> Self {
        Self::new(Internal::OptRequired(names.into()))
    }

    pub fn raise_sp_cmd_require(names: impl Into<String>) -> Self {
        Self::new(Internal::CmdRequired(names.into()))
    }

    pub fn raise_sp_not_found(hint: impl Into<String>) -> Self {
        Self::new(Internal::OptionNotFound(hint.into()))
    }

    pub fn raise_sp_extract(msg: impl Into<String>) -> Self {
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

#[macro_export]
macro_rules! raise_command {
    ($cmd:expr) => {
        $crate::Error::raise_command($cmd)
    };
}
