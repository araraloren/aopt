use std::convert::From;
use std::fmt::Display;

use crate::Str;

pub type Result<T> = std::result::Result<T, Error>;

/// Error string of [`Error`](crate::Error) type.
#[derive(Debug, Clone, Default)]
pub struct ErrorStr(String);

impl From<String> for ErrorStr {
    fn from(v: String) -> Self {
        Self(v)
    }
}

impl<'a> From<&'a String> for ErrorStr {
    fn from(v: &'a String) -> Self {
        Self(v.clone())
    }
}

impl<'a> From<&'a mut String> for ErrorStr {
    fn from(v: &'a mut String) -> Self {
        Self(v.clone())
    }
}

impl<'a> From<&'a str> for ErrorStr {
    fn from(v: &'a str) -> Self {
        Self(String::from(v))
    }
}

impl From<Str> for ErrorStr {
    fn from(v: Str) -> Self {
        Self(v.to_string())
    }
}

impl<'a> From<&'a Str> for ErrorStr {
    fn from(v: &'a Str) -> Self {
        Self(v.to_string())
    }
}

impl<'a> From<&'a mut Str> for ErrorStr {
    fn from(v: &'a mut Str) -> Self {
        Self(v.to_string())
    }
}

impl Display for ErrorStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[derive(Clone)]
pub enum Error {
    Null,

    Failure(ErrorStr),

    CustomError(ErrorStr),

    ArgMissingName(ErrorStr),

    ArgParsingError(ErrorStr),

    ArgMissingValue(ErrorStr),

    ConParsingFailed(ErrorStr),

    ConNoPOSIfCMDExists,

    ConOptionTypeError(ErrorStr),

    ConDeactivateStyleError(ErrorStr),

    ConMissingIndex(ErrorStr, ErrorStr),

    ConMissingField(ErrorStr, ErrorStr, ErrorStr),

    ConInvalidName(ErrorStr, ErrorStr),

    ConInvalidIndex(ErrorStr, ErrorStr),

    ConOptionAliasError(ErrorStr),

    ConParsingIndexFailed(ErrorStr, ErrorStr),

    SpExtractError(ErrorStr),

    SpMissingArgument(ErrorStr),

    SpOptForceRequired(ErrorStr),

    SpPOSForceRequired(ErrorStr),

    SpCMDForceRequired(ErrorStr),

    SpInvalidOptionName(ErrorStr),

    SpInvalidOptionValue(ErrorStr, ErrorStr),

    SpDeactivateStyleError(ErrorStr, bool),

    InvokeError(ErrorStr),
}

impl Default for Error {
    fn default() -> Self {
        Self::Null
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl Error {
    /// The error can be moitted if [`is_failure`](Error::is_failure) return true.
    ///
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Error::Failure(_)
                | Error::SpMissingArgument(_)
                | Error::SpPOSForceRequired(_)
                | Error::SpCMDForceRequired(_)
                | Error::SpInvalidOptionName(_)
                | Error::SpInvalidOptionValue(_, _)
                | Error::SpDeactivateStyleError(_, _)
                | Error::SpExtractError(_)
                | Error::InvokeError(_)
        )
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Error::Null)
    }

    /// Create Error::CustomError error
    pub fn raise_error<T: Into<ErrorStr>>(t: T) -> Self {
        Error::CustomError(t.into())
    }

    /// Create Error::Failure error
    pub fn raise_failure<T: Into<ErrorStr>>(t: T) -> Self {
        Error::Failure(t.into())
    }

    /// Create Error::ArgParsingError error
    pub fn arg_parsing_failed<T: Into<ErrorStr>>(t: T) -> Self {
        Self::ArgParsingError(t.into())
    }

    /// Create Error::ArgMissingName error
    pub fn arg_missing_name<T: Into<ErrorStr>>(t: T) -> Self {
        Self::ArgMissingName(t.into())
    }

    /// Create Error::ConNoPOSIfCMDExists error
    pub fn con_can_not_insert_pos() -> Self {
        Self::ConNoPOSIfCMDExists
    }

    /// Create Error::ConInvalidName error
    pub fn con_invalid_name<T: Into<ErrorStr>>(t: T, p: T) -> Self {
        Self::ConInvalidName(t.into(), p.into())
    }

    /// Create Error::ConMissingIndex error
    pub fn con_missing_index<T: Into<ErrorStr>>(t: T, p: T) -> Self {
        Self::ConMissingIndex(t.into(), p.into())
    }

    /// Create Error::ConMissingField error
    pub fn con_missing_field<T: Into<ErrorStr>>(f: T, t: T, p: T) -> Self {
        Self::ConMissingField(f.into(), t.into(), p.into())
    }

    /// Create Error::ConInvalidIndex error
    pub fn con_invalid_index<T: Into<ErrorStr>>(p: T, e: T) -> Self {
        Self::ConInvalidIndex(p.into(), e.into())
    }

    /// Create Error::ConParsingFailed error
    pub fn con_parsing_failed<T: Into<ErrorStr>>(t: T) -> Self {
        Self::ConParsingFailed(t.into())
    }

    /// Create Error::ConParsingIndexFailed error
    pub fn con_parsing_index_failed<T: Into<ErrorStr>>(t: T, e: T) -> Self {
        Self::ConParsingIndexFailed(t.into(), e.into())
    }

    /// Create Error::ConDeactivateStyleError error
    pub fn con_unsupport_deactivate_style<T: Into<ErrorStr>>(t: T) -> Self {
        Self::ConDeactivateStyleError(t.into())
    }

    /// Create Error::ConOptionTypeError error
    pub fn con_unsupport_option_type<T: Into<ErrorStr>>(t: T) -> Self {
        Self::ConOptionTypeError(t.into())
    }

    /// Create Error::ConOptionAliasError error
    pub fn con_invalid_option_alias<T: Into<ErrorStr>>(t: T) -> Self {
        Self::ConOptionAliasError(t.into())
    }

    /// Create Error::SpMissingArgument error
    pub fn sp_missing_argument<T: Into<ErrorStr>>(t: T) -> Self {
        Self::SpMissingArgument(t.into())
    }

    /// Create Error::SpPOSForceRequired error
    pub fn sp_pos_force_require<T: Into<ErrorStr>>(t: T) -> Self {
        Self::SpPOSForceRequired(t.into())
    }

    /// Create Error::SpOptForceRequired error
    pub fn sp_opt_force_require<T: Into<ErrorStr>>(t: T) -> Self {
        Self::SpOptForceRequired(t.into())
    }

    /// Create Error::SpCMDForceRequired error
    pub fn sp_cmd_force_require<T: Into<ErrorStr>>(t: T) -> Self {
        Self::SpCMDForceRequired(t.into())
    }

    /// Create Error::SpInvalidOptionName error
    pub fn sp_invalid_option_name<T: Into<ErrorStr>>(t: T) -> Self {
        Self::SpInvalidOptionName(t.into())
    }

    /// Create Error::SpInvalidOptionValue error
    pub fn sp_invalid_option_value<T: Into<ErrorStr>>(n: T, t: T) -> Self {
        Self::SpInvalidOptionValue(n.into(), t.into())
    }

    /// Create Error::SpDeactivateStyleError error
    pub fn sp_deactivate_style_error<T: Into<ErrorStr>>(t: T, support: bool) -> Self {
        Self::SpDeactivateStyleError(t.into(), support)
    }

    /// Create Error::SpExtractError error
    pub fn sp_extract_error<T: Into<ErrorStr>>(t: T) -> Self {
        Self::SpExtractError(t.into())
    }

    /// Create Error::InvokeError error
    pub fn invoke_error<T: Into<ErrorStr>>(t: T) -> Self {
        Self::InvokeError(t.into())
    }

    pub fn display(&self) -> String {
        match self {
            Error::Null => "Null".to_owned(),
            Error::Failure(opt) => opt.to_string(),
            Error::CustomError(opt) => opt.to_string(),
            Error::ArgMissingName(opt) => {
                format!("Syntax error! Missing option name: '{opt}'")
            }
            Error::ArgParsingError(opt) => {
                format!("Syntax error! Parsing failed: '{opt}'.")
            }
            Error::ArgMissingValue(opt) => {
                format!("Syntax error! Missing option value: '{opt}'.")
            }
            Error::ConNoPOSIfCMDExists => {
                "Can not have force required POS if CMD exists.".to_owned()
            }
            Error::ConOptionTypeError(value_type) => {
                format!("Not support option type '{value_type}'.")
            }
            Error::ConDeactivateStyleError(name) => {
                format!("Option '{name}' not support deactivate style.")
            }
            Error::ConInvalidName(name, msg) => {
                format!("Invalid name of option '{name}': {msg}")
            }
            Error::ConMissingIndex(name, value_type) => {
                format!("Syntax error! Missing index for option '{name}' with type '{value_type}'.")
            }
            Error::ConMissingField(field, name, value_type) => {
                format!(
                    "Syntax error! Missing `{field}` for option '{name}' with type '{value_type}'."
                )
            }
            Error::ConOptionAliasError(alias) => {
                format!("Invalid alias '{alias}', check the option prefix or name.")
            }
            Error::ConParsingIndexFailed(opt, err) => {
                format!("Syntax error! Parsing index '{opt}' failed: {err}.")
            }
            Error::ConParsingFailed(opt) => {
                format!("Syntax error! Parsing option string '{opt}' failed.")
            }
            Error::SpMissingArgument(opt) => {
                format!("Syntax error! Missing argument for option '{opt}'.")
            }
            Error::SpOptForceRequired(poss) => {
                format!("Option '{poss}' are force required.")
            }
            Error::SpPOSForceRequired(poss) => {
                format!("POS '{poss}' are force required.")
            }
            Error::SpCMDForceRequired(cmds) => {
                format!("CMD '{cmds}' are force required.")
            }
            Error::SpInvalidOptionName(name) => {
                format!("Can not find option '{name}'.")
            }
            Error::SpInvalidOptionValue(name, error) => {
                format!("Invalid option value for '{name}': {error}")
            }
            Error::SpDeactivateStyleError(msg, support) => {
                format!(
                    "Syntax error, option '{msg}' {} support deactivate style",
                    if *support { "only" } else { "not" }
                )
            }
            Error::SpExtractError(msg) => {
                format!("Extract error: {}", msg)
            }
            Error::InvokeError(msg) => msg.to_string(),
            Error::ConInvalidIndex(pattern, msg) => {
                format!("Syntax error, invalid index of '{pattern}': {msg}")
            }
        }
    }
}
