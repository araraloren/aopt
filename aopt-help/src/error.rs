use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub enum Error {
    Null,

    InvalidBlockName(String),

    DuplicatedBlockName(String),

    DuplicatedStoreName(String),

    DuplicatedCommandName(String),

    Error(String),
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
    pub fn raise(msg: String) -> Self {
        Self::Error(msg)
    }

    pub fn display(&self) -> String {
        match self {
            Error::Null => {
                panic!("Can not use Error::Null")
            }
            Error::InvalidBlockName(name) => {
                format!("Invalid block name {}", name)
            }
            Error::DuplicatedBlockName(name) => {
                format!("Duplicated block name {}", name)
            }
            Error::DuplicatedStoreName(name) => {
                format!("Duplicated store name {}", name)
            }
            Error::DuplicatedCommandName(name) => {
                format!("Duplicated command name {}", name)
            }
            Error::Error(msg) => msg.clone(),
        }
    }
}
