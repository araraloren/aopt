use crate::Error;
use crate::Str;

use super::ArgParser;

#[derive(Debug, Clone, Default)]
pub struct NullParser;

impl ArgParser for NullParser {
    type Output = Str;

    type Error = Error;

    /// Do nothing, return the `pattern` argument.
    fn parse(&mut self, pattern: Str, _prefixs: &[Str]) -> Result<Self::Output, Self::Error> {
        Ok(pattern)
    }
}
