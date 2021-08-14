use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Argument error: '{0}'")]
    FromArgumentError(#[from] ArgumentError),

    #[error("Construct error: '{0}'")]
    FromConstrutError(#[from] ConstructError),

    #[error("not support option type name `{0}`")]
    InvalidOptionTypeName(String),

    #[error("invalid value type for option `{0}`, found: {1}")]
    InvalidOptionValueType(String, String),

    #[error("can not invoke with callback type `{0}`")]
    InvalidCallbackType(String),

    #[error("the option `{0}` is force required")]
    ForceRequiredOption(String),

    #[error("option type is not support deactivate style: `{0}`")]
    NotSupportDeactivateStyle(String),

    #[error("pos option `{0}` index can not be null")]
    ForceRequiredOptionIndex(String),

    #[error("parse option value `{0}` failed: `{1}`")]
    ParseOptionValueFailed(String, String),

    #[error("option type `{0}` need an valid prefix")]
    NeedValidPrefix(&'static str),

    #[error("option `{0}` need an argument")]
    RequiredArgumentOfOption(String),

    #[error("inavlid return value type, except `{0}` found `{1}`")]
    InvalidReturnValueOfCallback(String, String),

    #[error("invalid option callback data: `{0}`")]
    InvalidOptionCallbackData(String),

    #[error("the option @{0} is force required: `{1}`")]
    ForceRequiredPostionOption(u64, String),

    // Special type mark the parsing failed !
    #[error("{0}")]
    ReportMatchFailed(String),

    #[error("{0}")]
    ReportError(String),
}

impl Error {
    pub fn is_special(&self) -> bool {
        match self {
            Self::ReportMatchFailed(_) => true,
            Self::ReportError(_) => true,
            _ => false,
        }
    }
}

pub fn report_an_error<T>(error_description: String) -> Result<T> {
    Err(Error::ReportError(format!(
        "report error: {}",
        error_description
    )))
}

pub fn report_match_failed<T>(error_description: String) -> Result<T> {
    Err(Error::ReportMatchFailed(format!(
        "match failed: {}",
        error_description
    )))
}

#[derive(Debug, thiserror::Error)]
pub enum ArgumentError {
    #[error("Failed parsing '{0}' as an option string")]
    ParsingFailed(String),

    #[error("Can not get sub-pattern({1} .. {2}) of '{0}'")]
    PatternAccessFailed(String, usize, usize),

    #[error("Syntax error! Missing an value after '=': '{0}'")]
    MissingValue(String),

    #[error("Syntax error! Missing option prefix: '{0}'")]
    MissingPrefix(String),

    #[error("Syntax error! Missing option name: '{0}'")]
    MissingName(String),
}

pub enum ParsingError {}

#[derive(Debug, thiserror::Error)]
pub enum ConstructError {
    #[error("Syntax error! Missing option type: '{0}'")]
    MissingOptionType(String),

    #[error("Syntax error! Missing option name: '{0}'")]
    MissingOptionName(String),

    #[error("Syntax error! Failed to parsing option string '{0}'")]
    ParsingFailed(String),

    #[error("Can not get sub-pattern({1} .. {2}) of '{0}'")]
    PatternAccessFailed(String, usize, usize),

    #[error("Syntax error! '{0}' parsing failed: {1}")]
    IndexParsingFailed(String, String),

    #[error("Option type '{0}' not support deactivate style")]
    NotSupportDeactivateStyle(String),

    #[error("Syntax error! Missing prefix for option type '{0}'")]
    MissingOptionPrefix(String),
}

pub enum SpecialError {}
