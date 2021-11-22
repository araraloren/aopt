use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid cmd name: '{0}'")]
    InvalidCmdName(String),

    #[error("Invalid section name: '{0}'")]
    InvalidSecName(String),

    #[error("Got io write error: '{0}'")]
    IOWriteError(#[from] std::io::Error),
}
