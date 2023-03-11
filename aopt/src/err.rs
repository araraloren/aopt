use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Null,
    Custom(String),
    InvalidArgName(String),
    UnexceptedPos,
    InvalidOptIndex { pattern: String, message: String },
    InvalidCreateStr { pattern: String, message: String },
    Failure(String),
    MissingOptValue(String),
    PosForceRequired(String),
    OptForceRequired(String),
    CmdForceRequired(String),
    OptionNotFound(String),
    ExtractValueError(String),
}

impl Default for Error {
    fn default() -> Self {
        Self::Null
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl Error {
    pub fn is_null(&self) -> bool {
        matches!(self, Error::Null)
    }

    /// The error can be moitted if [`is_failure`](Error::is_failure) return true.
    ///
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Error::Failure(_)
                | Error::ExtractValueError(_)
                | Error::OptionNotFound(_)
                | Error::CmdForceRequired(_)
                | Error::PosForceRequired(_)
                | Error::OptForceRequired(_)
                | Error::MissingOptValue(_)
        )
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn invalid_arg_name<T: Into<String>>(msg: T) -> Self {
        Self::InvalidArgName(msg.into())
    }

    /// No Pos@1 allowed if the option set has cmd.
    pub fn unexcepted_pos_if_has_cmd() -> Self {
        Self::UnexceptedPos
    }

    pub fn invalid_opt_index<T: Into<String>>(pattern: T, msg: T) -> Self {
        Self::InvalidOptIndex {
            pattern: pattern.into(),
            message: msg.into(),
        }
    }

    pub fn invalid_create_str<T: Into<String>>(pattern: T, msg: T) -> Self {
        Self::InvalidCreateStr {
            pattern: pattern.into(),
            message: msg.into(),
        }
    }

    pub fn raise_error(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    pub fn sp_missing_opt_value<T: Into<String>>(option: T) -> Self {
        Self::MissingOptValue(option.into())
    }

    pub fn sp_pos_force_require<T: Into<String>>(options: T) -> Self {
        Self::PosForceRequired(options.into())
    }

    pub fn sp_opt_force_require<T: Into<String>>(options: T) -> Self {
        Self::OptForceRequired(options.into())
    }

    pub fn sp_cmd_force_require<T: Into<String>>(options: T) -> Self {
        Self::CmdForceRequired(options.into())
    }

    pub fn sp_option_not_found<T: Into<String>>(option: T) -> Self {
        Self::OptionNotFound(option.into())
    }

    pub fn sp_raise_extract_error<T: Into<String>>(msg: T) -> Self {
        Self::ExtractValueError(msg.into())
    }

    pub fn raise_failure(message: impl Into<String>) -> Self {
        Self::Failure(message.into())
    }

    pub fn display(&self) -> String {
        match self {
            Error::Null => "Null { }".to_owned(),
            Error::Custom(message) => message.clone(),
            Error::InvalidArgName(msg) => {
                format!("Invalid argument name: {}", msg)
            }
            Error::UnexceptedPos => "Can not insert Pos@1 and Cmd both".to_owned(),
            Error::InvalidOptIndex { pattern, message } => {
                format!(
                    "Invalid index create string within `{}`: {}",
                    pattern, message
                )
            }
            Error::InvalidCreateStr { pattern, message } => {
                format!("Invalid option create string `{}`: {}", pattern, message)
            }
            Error::Failure(message) => message.clone(),
            Error::MissingOptValue(option) => {
                format!("Missing option value for `{}`", option)
            }
            Error::PosForceRequired(options) => {
                format!("Pos `{}` are force required", options)
            }
            Error::OptForceRequired(options) => {
                format!("Option `{}` are force required", options)
            }
            Error::CmdForceRequired(options) => {
                format!("Cmd `{}` are force required", options)
            }
            Error::OptionNotFound(option) => {
                format!("Can not find option `{}`", option)
            }
            Error::ExtractValueError(msg) => {
                format!("Extract value failed: {}", msg)
            }
        }
    }
}
