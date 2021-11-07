use super::parser::parse_argument;
use super::parser::DataKeeper;
use crate::err::ArgumentError;
use crate::err::Result;
use crate::Ustr;

#[derive(Debug, Clone, Default)]
pub struct Argument {
    pub current: Option<Ustr>,

    pub next: Option<Ustr>,

    data_keeper: DataKeeper,
}

impl Argument {
    pub fn new(current: Option<Ustr>, next: Option<Ustr>) -> Self {
        Self {
            current,
            next,
            ..Self::default()
        }
    }

    pub fn get_prefix(&self) -> &Option<Ustr> {
        &self.data_keeper.prefix
    }

    pub fn get_name(&self) -> &Option<Ustr> {
        &self.data_keeper.name
    }

    pub fn get_value(&self) -> &Option<Ustr> {
        &self.data_keeper.value
    }

    pub fn is_disabled(&self) -> bool {
        self.data_keeper.disable
    }

    pub fn parse(&mut self, prefix: &[Ustr]) -> Result<bool> {
        if let Some(pattern) = &self.current {
            self.data_keeper = parse_argument(pattern.clone(), prefix)?;

            // must have prefix and name
            if self.data_keeper.prefix.is_none() {
                return Err(ArgumentError::MissingPrefix(pattern.to_string()).into());
            }
            if self.data_keeper.name.is_none() {
                return Err(ArgumentError::MissingName(pattern.to_string()).into());
            }
            return Ok(true);
        }
        Ok(false)
    }
}
