use super::parser::parse_argument;
use super::parser::DataKeeper;
use crate::err::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct Argument<'pre> {
    pub current: Option<String>,

    pub next: Option<String>,

    data_keeper: DataKeeper<'pre>,
}

impl<'pre> Argument<'pre> {
    pub fn new(current: Option<String>, next: Option<String>) -> Self {
        Self {
            current,
            next,
            ..Self::default()
        }
    }

    pub fn get_prefix(&self) -> Option<&'pre String> {
        self.data_keeper.prefix.clone()
    }

    pub fn get_name(&self) -> Option<&String> {
        self.data_keeper.name.as_ref()
    }

    pub fn get_value(&self) -> Option<&String> {
        self.data_keeper.value.as_ref()
    }

    pub fn is_disabled(&self) -> bool {
        self.data_keeper.disable
    }

    #[cfg(not(feature = "async"))]
    pub fn parse(&mut self, prefix: &'pre [String]) -> Result<bool> {
        if let Some(pattern) = &self.current {
            self.data_keeper = parse_argument(pattern.as_ref(), prefix)?;

            // must have prefix and name
            if self.data_keeper.prefix.is_some() {
                if self.data_keeper.name.is_some() {
                    return Ok(true);
                }
            }
        }
        Err(Error::NotOptionArgument)
    }

    #[cfg(feature = "async")]
    pub async fn parse(&mut self, prefix: &'pre [String]) -> Result<bool> {
        if let Some(pattern) = &self.current {
            self.data_keeper = parse_argument(pattern.as_ref(), prefix)?;

            // must have prefix and name
            if self.data_keeper.prefix.is_some() {
                if self.data_keeper.name.is_some() {
                    return Ok(true);
                }
            }
        }
        Err(Error::NotAOptionArgument)
    }
}
