use std::fmt::Display;
use thiserror::Error;
use ustr::Ustr;

pub type Result<T> = std::result::Result<T, Error>;

/// Error string of [`Error`](crate::err::Error) type.
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

impl From<Ustr> for ErrorStr {
    fn from(v: Ustr) -> Self {
        Self(v.to_string())
    }
}

impl<'a> From<&'a Ustr> for ErrorStr {
    fn from(v: &'a Ustr) -> Self {
        Self(v.to_string())
    }
}

impl<'a> From<&'a mut Ustr> for ErrorStr {
    fn from(v: &'a mut Ustr) -> Self {
        Self(v.to_string())
    }
}

impl Display for ErrorStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Argument error: {0}")]
    FromArgumentError(#[from] ArgumentError),

    #[error("Construct error: {0}")]
    FromConstrutError(#[from] ConstructError),

    #[error("Special error: {0}")]
    FromSpecialError(#[from] SpecialError),

    #[error("{0}")]
    CustomError(ErrorStr),
}

impl Error {
    pub fn is_special(&self) -> bool {
        matches!(self, Self::FromSpecialError(_))
    }

    /// Create Error::CustomError error
    pub fn raise_error<T: Into<ErrorStr>>(t: T) -> Self {
        Error::CustomError(t.into())
    }

    /// Create SpecialError::CustomFailure error
    pub fn raise_failure<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::CustomFailure(t.into()).into()
    }

    /// Create ArgumentError::ParsingFailed error
    pub fn arg_parsing_failed<T: Into<ErrorStr>>(t: T) -> Self {
        ArgumentError::ParsingFailed(t.into()).into()
    }

    /// Create ArgumentError::PatternOutOfRange error
    pub fn arg_pattern_out_of_range<T: Into<ErrorStr>>(t: T, start: usize, end: usize) -> Self {
        ArgumentError::PatternOutOfRange(t.into(), start, end).into()
    }

    /// Create ArgumentError::MissingValue error
    pub fn arg_missing_value<T: Into<ErrorStr>>(t: T) -> Self {
        ArgumentError::MissingValue(t.into()).into()
    }

    /// Create ArgumentError::MissingPrefix error
    pub fn arg_missing_prefix<T: Into<ErrorStr>>(t: T) -> Self {
        ArgumentError::MissingPrefix(t.into()).into()
    }

    /// Create ArgumentError::MissingName error
    pub fn arg_missing_name<T: Into<ErrorStr>>(t: T) -> Self {
        ArgumentError::MissingName(t.into()).into()
    }

    /// Create ArgumentError::UnwrapError error
    pub fn arg_unwrap_value_failed<T: Into<ErrorStr>>(t: T) -> Self {
        ArgumentError::UnwrapValueFailed(t.into()).into()
    }

    /// Create ConstructError::ParsingValueFailed error
    pub fn opt_parsing_value_failed<T: Into<ErrorStr>>(t: T, e: T) -> Self {
        ConstructError::ParsingValueFailed(t.into(), e.into()).into()
    }

    /// Create ConstructError::InvalidReturnValueOfCallback error
    pub fn opt_invalid_ret_value<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::InvalidRetValueOfCallback(t.into()).into()
    }

    /// Create ConstructError::CanNotInsertPOSIfCMDExists error
    pub fn opt_can_not_insert_pos() -> Self {
        ConstructError::CanNotInsertPOSIfCMDExists.into()
    }

    /// Create ConstructError::MissingOptionType error
    pub fn opt_missing_type<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::MissingOptionType(t.into()).into()
    }

    /// Create ConstructError::MissingOptionName error
    pub fn opt_missing_name<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::MissingOptionName(t.into()).into()
    }

    /// Create ConstructError::MissingOptionPrefix error
    pub fn opt_missing_prefix<T: Into<ErrorStr>>(t: T, p: T) -> Self {
        ConstructError::MissingOptionPrefix(t.into(), p.into()).into()
    }

    /// Create ConstructError::MissingNonOptionIndex error
    pub fn opt_missing_index<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::MissingNonOptionIndex(t.into()).into()
    }

    /// Create ConstructError::ParsingConstructorFailed error
    pub fn opt_parsing_constructor_failed<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::ParsingConstructorFailed(t.into()).into()
    }

    /// Create ConstructError::PatternOutOfRange error
    pub fn opt_pattern_out_of_range<T: Into<ErrorStr>>(t: T, start: usize, end: usize) -> Self {
        ConstructError::PatternOutOfRange(t.into(), start, end).into()
    }

    /// Create ConstructError::IndexParsingFailed error
    pub fn opt_parsing_index_failed<T: Into<ErrorStr>>(t: T, e: T) -> Self {
        ConstructError::IndexParsingFailed(t.into(), e.into()).into()
    }

    /// Create ConstructError::NotSupportDeactivateStyle error
    pub fn opt_unsupport_deactivate_style<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::NotSupportDeactivateStyle(t.into()).into()
    }

    /// Create ConstructError::NotSupportCallbackType error
    pub fn opt_unsupport_callback_type<T: Into<ErrorStr>>(t: T, v: T) -> Self {
        ConstructError::NotSupportCallbackType(t.into(), v.into()).into()
    }

    /// Create ConstructError::NotSupportOptionType error
    pub fn opt_unsupport_option_type<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::NotSupportOptionType(t.into()).into()
    }

    /// Create ConstructError::InvalidOptionAlias error
    pub fn opt_invalid_alias<T: Into<ErrorStr>>(t: T) -> Self {
        ConstructError::InvalidOptionAlias(t.into()).into()
    }

    /// Create SpecialError::OptionForceRequired error
    pub fn sp_option_force_require<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::OptionForceRequired(t.into()).into()
    }

    /// Create SpecialError::MissingArgumentForOption error    
    pub fn sp_missing_argument<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::MissingArgumentForOption(t.into()).into()
    }

    /// Create SpecialError::InvalidArgumentForOption error
    pub fn sp_invalid_argument<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::InvalidArgumentForOption(t.into()).into()
    }

    /// Create SpecialError::POSForceRequired error
    pub fn sp_pos_force_require<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::POSForceRequired(t.into()).into()
    }

    /// Create SpecialError::CMDForceRequired error
    pub fn sp_cmd_force_require<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::CMDForceRequired(t.into()).into()
    }

    /// Create SpecialError::InvalidOptionName error
    pub fn sp_invalid_option_name<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::InvalidOptionName(t.into()).into()
    }

    /// Create SpecialError::NotSupportDeactivateStyle error
    pub fn sp_unsupport_deactivate_style<T: Into<ErrorStr>>(t: T) -> Self {
        SpecialError::NotSupportDeactivateStyle(t.into()).into()
    }
}

pub fn create_error(error_description: String) -> Error {
    Error::CustomError(error_description.into())
}

pub fn create_failure(msg: String) -> SpecialError {
    SpecialError::CustomFailure(msg.into())
}

/// Errors of parsing command line item.
#[derive(Debug, thiserror::Error)]
pub enum ArgumentError {
    #[error("Failed parsing `{0}` as an option string")]
    ParsingFailed(ErrorStr),

    #[error("Can not get sub-pattern({1} .. {2}) of `{0}`")]
    PatternOutOfRange(ErrorStr, usize, usize),

    #[error("Syntax error! Missing an value after '=': {0}")]
    MissingValue(ErrorStr),

    #[error("Syntax error! Missing option prefix: {0}")]
    MissingPrefix(ErrorStr),

    #[error("Syntax error! Missing option name: {0}")]
    MissingName(ErrorStr),

    #[error("Can not unwrap `{0}` from Argument")]
    UnwrapValueFailed(ErrorStr),
}

/// Errors of option construct and parsing.
#[derive(Debug, thiserror::Error)]
pub enum ConstructError {
    #[error("Syntax error! Missing option type: {0}")]
    MissingOptionType(ErrorStr),

    #[error("Syntax error! Missing option name: {0}")]
    MissingOptionName(ErrorStr),

    #[error("Syntax error! Failed to parsing constructor: {0}")]
    ParsingConstructorFailed(ErrorStr),

    #[error("Can not get sub-pattern({1} .. {2}) of `{0}`")]
    PatternOutOfRange(ErrorStr, usize, usize),

    #[error("Syntax error! Invalid index `{0}`: {1}")]
    IndexParsingFailed(ErrorStr, ErrorStr),

    #[error("Option `{0}` not support deactivate style")]
    NotSupportDeactivateStyle(ErrorStr),

    #[error("Syntax error! Missing prefix for option `{0}` with type `{1}`")]
    MissingOptionPrefix(ErrorStr, ErrorStr),

    #[error("Syntax error! Missing Non-Option index: `{0}`")]
    MissingNonOptionIndex(ErrorStr),

    #[error("Option `{0}` not support callback type `{1}`")]
    NotSupportCallbackType(ErrorStr, ErrorStr),

    #[error("Not support option type `{0}`")]
    NotSupportOptionType(ErrorStr),

    #[error("Invalid alias `{0}`, check the option prefix or name")]
    InvalidOptionAlias(ErrorStr),

    #[error("Failed parsing `{0}` as option value: {1}")]
    ParsingValueFailed(ErrorStr, ErrorStr),

    #[error("Invalid callback return value type: `{0}`")]
    InvalidRetValueOfCallback(ErrorStr),

    #[error("Can not have force required POS if CMD exists")]
    CanNotInsertPOSIfCMDExists,
}

/// Special error using for parser.
///
/// When using [`getopt!`](crate::getopt!) for multiple [`Parser`](crate::parser::Parser),
/// current error will not treat as error until last Parser.
#[derive(Debug, thiserror::Error)]
pub enum SpecialError {
    #[error("Option `{0}` is force required")]
    OptionForceRequired(ErrorStr),

    #[error("Missing argument for option `{0}`")]
    MissingArgumentForOption(ErrorStr),

    #[error("Invalid value for option `{0}`")]
    InvalidArgumentForOption(ErrorStr),

    #[error("POS `{0}` is force required")]
    POSForceRequired(ErrorStr),

    #[error("CMD `{0}` is force required")]
    CMDForceRequired(ErrorStr),

    #[error("Invalid option name: `{0}`")]
    InvalidOptionName(ErrorStr),

    #[error("Can not disable option which not support deactivate style: `{0}`")]
    NotSupportDeactivateStyle(ErrorStr),

    #[error("{0}")]
    CustomFailure(ErrorStr),
}
