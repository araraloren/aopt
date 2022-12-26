use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use crate::args::ArgParser;
use crate::astr;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;

/// Return an [`OsString`] with the prefix removed if the prefix exists.
fn strip_prefix(str: &OsStr, prefix: &str) -> Option<OsString> {
    let enc = str.encode_wide();
    let mut pre = prefix.encode_utf16();
    let mut ret = Vec::with_capacity(str.len().saturating_sub(prefix.len()));

    for ori in enc {
        match pre.next() {
            Some(ch) => {
                // skip the character in prefix
                if ch != ori {
                    return None;
                }
            }
            None => {
                // add the left character into return value
                ret.push(ori);
            }
        }
    }
    Some(OsString::from_wide(&ret))
}

/// Split the string on the first occurrence of `ch`.
/// Returns prefix before delimiter and suffix after delimiter.
fn split_once(str: &OsStr, ch: char) -> Option<(OsString, OsString)> {
    let enc = str.encode_wide();
    let mut buf = [0; 1];
    let sep = ch.encode_utf16(&mut buf);
    let enc = enc.collect::<Vec<u16>>();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .map(|(idx, _)| {
            (
                OsString::from_wide(&enc[0..idx]),
                OsString::from_wide(&enc[idx + 1..]),
            )
        })
}

pub trait AOsStrExt {
    fn strip_prefix(&self, prefix: &str) -> Option<OsString>;

    fn split_once(&self, ch: char) -> Option<(OsString, OsString)>;
}

impl AOsStrExt for OsStr {
    fn strip_prefix(&self, prefix: &str) -> Option<OsString> {
        strip_prefix(self, prefix)
    }

    fn split_once(&self, ch: char) -> Option<(OsString, OsString)> {
        split_once(self, ch)
    }
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
/// # use aopt::Arc;
/// # use aopt::RawVal;
/// # use aopt::args::ArgParser;
/// #
/// # fn main() -> Result<(), Error> {
///     {// parse option with value
///         let output = RawVal::from("--foo=32").parse_arg()?;
///
///         assert_eq!(output.name, Some(astr("--foo")));
///         assert_eq!(output.value, Some(Arc::new(RawVal::from("32"))));
///     }
///     {// parse boolean option
///         let output = RawVal::from("--/bar").parse_arg()?;
///
///         assert_eq!(output.name, Some(astr("--/bar")));
///         assert_eq!(output.value, None);
///     }
///     {// parse other string
///         let output = RawVal::from("-=bar").parse_arg()?;
///
///         assert_eq!(output.name, Some(astr("-")));
///         assert_eq!(output.value, Some(Arc::new(RawVal::from("bar"))));
///     }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CLOpt {
    pub name: Option<Str>,

    pub value: Option<Arc<RawVal>>,
}

impl CLOpt {
    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn value(&self) -> Option<&Arc<RawVal>> {
        self.value.as_ref()
    }
}

const EQUAL: char = '=';

#[cfg(not(feature = "utf8"))]
impl ArgParser for RawVal {
    type Output = CLOpt;

    type Error = Error;

    fn parse_arg(&self) -> Result<Self::Output, Self::Error> {
        if let Some((name, value)) = self.split_once(EQUAL) {
            // - convert the name to &str, the name must be valid utf8
            let name = name
                .to_str()
                .ok_or_else(|| {
                    Error::arg_missing_name(format!("Name must be valid utf8: {:?}", name))
                })?
                .trim();

            if name.is_empty() {
                return Err(Error::arg_missing_name("Name can not be empty"));
            }
            return Ok(Self::Output {
                name: Some(astr(name)),
                value: Some(Arc::new(value.into())),
            });
        } else {
            let name = self
                .to_str()
                .ok_or_else(|| {
                    Error::arg_parsing_failed(format!(
                        "Not a valid option setting string: {:?}",
                        self
                    ))
                })?
                .trim();

            return Ok(Self::Output {
                name: Some(Str::from(name)),
                value: None,
            });
        };
    }
}

#[cfg(feature = "utf8")]
impl ArgParser for RawVal {
    type Output = CLOpt;

    type Error = Error;

    fn parse_arg(&self) -> Result<Self::Output, Self::Error> {
        if let Some((name, value)) = self.split_once(EQUAL) {
            let name = name.trim();

            if name.is_empty() {
                return Err(Error::arg_missing_name("Name can not be empty"));
            }

            Ok(Self::Output {
                name: Some(astr(name)),
                value: Some(Arc::new(value.into())),
            })
        } else {
            Ok(Self::Output {
                name: Some(astr(self.as_str())),
                value: None,
            })
        }
    }
}
