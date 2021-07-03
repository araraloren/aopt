use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed parse option string `{0}`")]
    InvalidOptionCreateString(String),

    #[error("argument looks like not a option setting")]
    NotOptionArgument,

    #[error("failed get string with range: {:?} .. {:?}", beg, end)]
    InvalidStringRange { beg: usize, end: usize },

    #[error("option string with '=' need an value after it: `{0}`")]
    RequireValueForArgument(String),

    #[error("invalid option index value: `{0}`")]
    InavlidOptionIndexValue(String),

    #[error("not support option type name `{0}`")]
    InvalidOptionTypeName(String),

    #[error("can not invoke with callback type `{0}`")]
    InvalidCallbackType(String),

    #[error("the option `{0}` is force required")]
    ForceRequiredOption(String),

    #[error("option type is not support deactivate style: `{0}`")]
    NotSupportDeactivateStyle(String),

    #[error("parse option value `{0}` failed: `{1}`")]
    ParseOptionValueFailed(String, String),

    #[error("option type `{0}` need an valid prefix")]
    NeedValidPrefix(&'static str),
}
