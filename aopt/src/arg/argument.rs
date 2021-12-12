use ustr::Ustr;

use super::parser::parse_argument;
use super::parser::DataKeeper;
use crate::err::ArgumentError;
use crate::err::Result;

/// Argument hold current and next item of command line arguments.
///
/// When parsing the command line option need an argument.
/// The argument of option may embedded in itself.
/// Or we need consume next item as argument of the option.
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

    /// Return true if the option contain deactivate style symbol '/'
    pub fn is_disabled(&self) -> bool {
        self.data_keeper.disable
    }

    /// Call [`parse_argument`] parse the command line item with given prefix.
    ///
    /// # Returns
    ///
    /// Will save the [`DataKeeper`] to self and return `Ok(true)` when successed.
    /// Return `Ok(false)` when current item is [`None`].
    ///
    /// # Errors
    ///
    /// Will return [`ArgumentError::MissingPrefix`] when the result not have a valid prefix.
    /// Or return [`ArgumentError::MissingName`] when the result not have a valid name.
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
