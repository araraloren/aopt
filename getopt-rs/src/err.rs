use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Argument error: '{0}'")]
    FromArgumentError(#[from] ArgumentError),

    #[error("Construct error: '{0}'")]
    FromConstrutError(#[from] ConstructError),

    #[error("Parser error: '{0}'")]
    FromParserError(#[from] ParserError),

    #[error("Special error: '{0}'")]
    FromSpecialError(#[from] SpecialError),

    #[error("{0}")]
    CustomError(String),
}

impl Error {
    pub fn is_special(&self) -> bool {
        match self {
            Self::FromSpecialError(_) => true,
            _ => false,
        }
    }
}

pub fn create_error(error_description: String) -> Error {
    Error::CustomError(error_description)
}

pub fn create_failure(msg: String) -> SpecialError {
    SpecialError::CustomFailure(msg)
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

    #[error("Can not unwrap '{0}' from Argument")]
    UnwrapError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Not support option type '{0}'")]
    NotSupportOptionType(String),

    #[error("Failed parsing '{0}': '{1}'")]
    ParsingValueFailed(String, String),

    #[error("Invalid callback return value type: '{0}'")]
    InvalidReturnValueOfCallback(String),
}

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

    #[error("Syntax error! Missing Non-Option index: '{0}'")]
    MissingNonOptionIndex(String),

    #[error("Option '{0} not support callback type '{1}'")]
    NotSupportCallbackType(String, String),

    #[error("Not support option type '{0}'")]
    NotSupportOptionType(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SpecialError {
    #[error("Option '{0}' is force required")]
    OptionForceRequired(String),

    #[error("Missing argument for option '{0}'")]
    MissingArgumentForOption(String),

    #[error("Invalid value for option '{0}'")]
    InvalidArgumentForOption(String),

    #[error("POS @{0} is force required: `{1}`")]
    POSForceRequired(u64, String),

    #[error("{0}")]
    CustomFailure(String),
}
