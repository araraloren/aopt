use std::ffi::OsStr;

use crate::Error;
use crate::Str;

#[cfg_attr(windows, path = "osstr_win.rs")]
#[cfg_attr(unix, path = "osstr_unix.rs")]
pub(crate) mod osstr_ext;

pub(crate) mod args;
pub(crate) mod parser_cl;

pub use self::args::Args;
pub use self::osstr_ext::AOsStrExt;
pub use self::osstr_ext::CLOpt;

/// Argument parser using for parse command line arguments.
pub trait ArgParser {
    type Output;
    type Error: Into<Error>;

    fn parse(&self, prefixs: &[Str]) -> Result<Self::Output, Self::Error>;
}
