use std::ffi::OsStr;

use crate::Error;
use crate::Str;

#[cfg_attr(windows, path = "osstr_win.rs")]
#[cfg_attr(unix, path = "osstr_unix.rs")]
pub(crate) mod osstr_ext;

pub(crate) mod args;
pub(crate) mod parser_cl;
pub(crate) mod parser_null;

pub use self::args::OptsIter;
pub use self::parser_cl::CLOpt;
pub use self::parser_cl::CLOptParser;
pub use self::parser_null::NullParser;

/// Argument parser using for parse command line arguments.
pub trait ArgParser {
    type Output;
    type Error: Into<Error>;

    fn parse(&mut self, pattern: &OsStr, prefixs: &[Str]) -> Result<Self::Output, Self::Error>;
}
