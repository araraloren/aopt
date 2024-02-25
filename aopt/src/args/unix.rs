use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use crate::args::ArgParser;
use crate::astr;
use crate::AStr;
use crate::Error;
use crate::RawVal;

pub fn strip_prefix<'a>(str: &'a OsStr, prefix: &str) -> Option<&'a OsStr> {
    let enc = str.as_bytes();
    let pre = prefix.as_bytes();

    enc.strip_prefix(pre).map(OsStr::from_bytes)
}

pub fn split_once(str: &OsStr, ch: char) -> Option<(&OsStr, &OsStr)> {
    let enc = str.as_bytes();
    let mut buf = [0; 1];
    let sep = ch.encode_utf8(&mut buf).as_bytes();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .map(|(idx, _)| {
            (
                OsStr::from_bytes(&enc[0..idx]),
                OsStr::from_bytes(&enc[idx + 1..]),
            )
        })
}

/// Parse the input command line item with given regexs, return an [`CLOpt`].
///
/// The struct of the input option string are:
///
/// ```!
/// [--/option][=][value]
///        |    |    |
///        |    |    |
///        |    |    The value part, it is optional.
///        |    |
///        |    The delimiter of option name and value.
///        |    
///        The option name part, it must be provide by user.
/// ```
///
/// # Example
///
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use aopt::astr;
/// # use aopt::RawVal;
/// # use aopt::args::ArgParser;
/// #
/// # fn main() -> Result<(), Error> {
///     {// parse option with value
///         let output = RawVal::from("--foo=32").parse_arg()?;
///
///         assert_eq!(output.name, astr("--foo"));
///         assert_eq!(output.value, Some(RawVal::from("32")));
///     }
///     {// parse boolean option
///         let output = RawVal::from("--/bar").parse_arg()?;
///
///         assert_eq!(output.name, astr("--/bar"));
///         assert_eq!(output.value, None);
///     }
///     {// parse other string
///         let output = RawVal::from("-=bar").parse_arg()?;
///
///         assert_eq!(output.name, astr("-"));
///         assert_eq!(output.value, Some(RawVal::from("bar")));
///     }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CLOpt {
    pub name: AStr,

    pub value: Option<RawVal>,
}

const EQUAL: char = '=';

impl ArgParser for RawVal {
    type Output = CLOpt;

    type Error = Error;

    fn parse_arg(&self) -> Result<Self::Output, Self::Error> {
        if let Some((name, value)) = split_once(self, EQUAL) {
            let name = name
                .to_str()
                .ok_or_else(|| {
                    Error::invalid_arg_name(format!(
                        "failed convert argument name `{}` to str",
                        self
                    ))
                })?
                .trim();
            if name.is_empty() {
                return Err(Error::invalid_arg_name("argument name can not be empty"));
            }

            Ok(Self::Output {
                name: astr(name),
                value: Some(value.into()),
            })
        } else {
            let name = self
                .to_str()
                .ok_or_else(|| {
                    Error::invalid_arg_name(format!(
                        "failed convert argument name `{}` to str",
                        self
                    ))
                })?
                .trim();

            Ok(Self::Output {
                name: astr(name),
                value: None,
            })
        }
    }
}
