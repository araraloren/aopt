use crate::Error;
use crate::Str;

pub(crate) mod args;
pub(crate) mod parser_cl;
pub(crate) mod parser_null;

pub use self::args::Args;
pub use self::parser_cl::CLOpt;
pub use self::parser_cl::CLOptParser;
pub use self::parser_null::NullParser;

/// Argument parser using for parse command line arguments.
pub trait ArgParser {
    type Output;
    type Error: Into<Error>;

    fn parse(&mut self, pattern: Str, prefixs: &[Str]) -> Result<Self::Output, Self::Error>;
}
