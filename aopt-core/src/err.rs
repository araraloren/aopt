use std::ffi::OsStr;
use std::fmt::Display;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::ops::Deref;
use std::thread::AccessError;

use crate::str::display_of_osstr;
use crate::Uid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    MissingValue,

    PosRequired,

    OptRequired,

    CmdRequired,

    OptionNotFound,

    ExtractValue,

    RawValParse,

    Arg,

    IndexParse,

    CreateStrParse,

    Failure,

    Error,

    NoParserMatched,

    UnexceptedPos,

    ThreadLocalAccess,
}

impl Kind {
    const fn desp(&self) -> Option<&'static str> {
        match self {
            Kind::UnexceptedPos => Some("can not insert Pos@1 if Cmd exist"),
            Kind::ThreadLocalAccess => Some("failed access thread local variable"),
            Kind::NoParserMatched => Some("all parser passed to `getopt!` match failed"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    uid: Option<Uid>,

    kind: Kind,

    desp: Option<String>,

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
        let desp = self.desp.as_deref().or(self.kind.desp());

        assert!(
            desp.is_some(),
            "need description for error `{:?}`",
            self.kind
        );

        if let Some(uid) = self.uid {
            write!(f, "{} (uid = {})", desp.unwrap(), uid)
        } else {
            write!(f, "{}", desp.unwrap())
        }
    }
}

impl Error {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            uid: None,
            desp: None,
            cause: None,
        }
    }

    pub fn cause(self, error: Self) -> Self {
        error.cause_by(self)
    }

    pub fn cause_by(mut self, cause_by: Self) -> Self {
        let _ = self.cause.insert(Box::new(cause_by));
        self
    }

    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = Some(uid);
        self
    }

    pub fn with_desp(mut self, desp: String) -> Self {
        self.desp = Some(desp);
        self
    }

    pub fn uid(&self) -> Option<Uid> {
        self.uid
    }

    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    pub fn caused_by(&self) -> Option<&Error> {
        self.cause.as_deref()
    }

    /// The error can be moitted if [`is_failure`](Error::is_failure) return true.
    pub fn is_failure(&self) -> bool {
        let kind = &self.kind;

        matches!(
            kind,
            Kind::RawValParse
                | Kind::Failure
                | Kind::ExtractValue
                | Kind::OptionNotFound
                | Kind::CmdRequired
                | Kind::PosRequired
                | Kind::OptRequired
                | Kind::MissingValue
        )
    }

    /// No Pos@1 allowed if the option set has cmd.
    pub fn unexcepted_pos() -> Self {
        Self::new(Kind::UnexceptedPos)
    }

    pub fn thread_local_access() -> Self {
        Self::new(Kind::ThreadLocalAccess)
    }

    pub fn no_parser_matched() -> Self {
        Self::new(Kind::NoParserMatched)
    }

    pub fn from<E: std::error::Error + Display>(error: E) -> Self {
        Self::raise_error(error.to_string())
    }

    pub fn arg(arg: impl Into<String>, hint: impl Into<String>) -> Self {
        let desp = format!("invalid argument `{}`: {}", arg.into(), hint.into());

        Self::new(Kind::Arg).with_desp(desp)
    }

    pub fn sp_rawval(val: Option<&OsStr>, hint: impl Into<String>) -> Self {
        let desp = format!("invalid value `{}`: {}", display_of_osstr(val), hint.into());

        Self::new(Kind::RawValParse).with_desp(desp)
    }

    pub fn index_parse(pat: impl Into<String>, hint: impl Into<String>) -> Self {
        let desp = format!("invalid index string `{}`: {}", pat.into(), hint.into());

        Self::new(Kind::IndexParse).with_desp(desp)
    }

    pub fn create_str(pat: impl Into<String>, hint: impl Into<String>) -> Self {
        let desp = format!(
            "invalid option create string `{}`: {}",
            pat.into(),
            hint.into()
        );

        Self::new(Kind::CreateStrParse).with_desp(desp)
    }

    pub fn raise_error(msg: impl Into<String>) -> Self {
        Self::new(Kind::Error).with_desp(msg.into())
    }

    pub fn raise_failure(msg: impl Into<String>) -> Self {
        Self::new(Kind::Failure).with_desp(msg.into())
    }

    pub fn sp_missing_value(name: impl Into<String>) -> Self {
        let desp = format!("missing value for option `{}`", name.into());

        Self::new(Kind::MissingValue).with_desp(desp)
    }

    pub fn sp_pos_require<S: Into<String>>(names: Vec<S>) -> Self {
        let names: Vec<_> = names.into_iter().map(Into::into).collect();
        let desp = match names.len() {
            1 => {
                format!("positional `{}` is force required", names[0])
            }
            _ => {
                format!("positional `{}` are force required", names.join(", "))
            }
        };

        Self::new(Kind::PosRequired).with_desp(desp)
    }

    pub fn sp_opt_require<S: Into<String>>(names: Vec<S>) -> Self {
        let names: Vec<_> = names.into_iter().map(Into::into).collect();
        let desp = match names.len() {
            1 => {
                format!("option `{}` is force required", names[0])
            }
            _ => {
                format!("option `{}` are force required", names.join(", "))
            }
        };

        Self::new(Kind::OptRequired).with_desp(desp)
    }

    pub fn sp_cmd_require<S: Into<String>>(names: Vec<S>) -> Self {
        let names: Vec<_> = names.into_iter().map(Into::into).collect();
        let desp = match names.len() {
            1 => {
                format!("command `{}` is force required", names[0])
            }
            _ => {
                format!("command `{}` are force required", names.join(", "))
            }
        };

        Self::new(Kind::CmdRequired).with_desp(desp)
    }

    pub fn sp_not_found(name: impl Into<String>) -> Self {
        let desp = format!("can not find option `{}`", name.into());

        Self::new(Kind::OptionNotFound).with_desp(desp)
    }

    pub fn sp_extract(msg: impl Into<String>) -> Self {
        let desp = format!("extract value failed: `{}`", msg.into());

        Self::new(Kind::ExtractValue).with_desp(desp)
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
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::Error::raise_error(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! failure {
    ($($arg:tt)*) => {
        $crate::Error::raise_failure(format!($($arg)*))
    };
}
