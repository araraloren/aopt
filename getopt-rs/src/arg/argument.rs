
use crate::str::Str;
use crate::err::Result;

use super::parser::DataKeeper;
use super::parser::parse_argument;

#[derive(Debug, Clone, Default)]
pub struct Argument<'str, 'nv, 'pre> {
    pub current: Option<Str<'str>>,

    pub next: Option<Str<'str>>,
    
    data_keeper: DataKeeper<'nv, 'pre>,
}

impl<'str, 'nv, 'pre> Argument<'str, 'nv, 'pre> {
    pub fn new(current: Option<Str<'str>>, next: Option<Str<'str>>) -> Self {
        Self {
            current,
            next,
            .. Self::default()
        }
    }

    pub fn get_prefix(&self) -> Option<&Str<'pre>> {
        self.data_keeper.prefix.as_ref()
    }

    pub fn get_name(&self) -> Option<&Str<'nv>> {
        self.data_keeper.name.as_ref()
    }

    pub fn get_value(&self) -> Option<&Str<'nv>> {
        self.data_keeper.value.as_ref()
    }

    pub fn is_disabled(&self) -> bool {
        self.data_keeper.disable
    }

    #[cfg(not(feature="async"))]
    pub fn parse<'x>(&mut self, prefix: &'pre Vec<Str<'x>>) -> Result<bool> {
        if let Some(pattern) = &self.current {
            self.data_keeper = parse_argument(pattern.as_ref(), prefix)?;
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    #[cfg(feature="async")]
    pub async fn parse(&mut self, prefix: &Vec<Str<'b>>) -> Result<bool> {
        if let Some(pattern) = &self.current {
            self.data_keeper = parse_argument(pattern.as_ref(), prefix)?;
            Ok(true)
        }
        else {
            Ok(false)
        }
    }
}