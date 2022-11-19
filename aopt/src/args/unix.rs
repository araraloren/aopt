use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use crate::args::ArgParser;
use crate::astr;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;

fn strip_prefix<'a>(str: &'a OsStr, prefix: &str) -> Option<&'a OsStr> {
    let enc = str.as_bytes();
    let pre = prefix.as_bytes();

    enc.strip_prefix(pre)
        .and_then(|v| Some(OsStr::from_bytes(v)))
}

fn split_once(str: &OsStr, ch: char) -> Option<(&OsStr, &OsStr)> {
    let enc = str.as_bytes();
    let mut buf = [0; 1];
    let sep = ch.encode_utf8(&mut buf).as_bytes();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .and_then(|(idx, _)| {
            Some((
                OsStr::from_bytes(&enc[0..idx]),
                OsStr::from_bytes(&enc[idx + 1..]),
            ))
        })
}

pub trait AOsStrExt {
    fn strip_prefix(&self, prefix: &str) -> Option<&OsStr>;

    fn split_once(&self, ch: char) -> Option<(&OsStr, &OsStr)>;
}

impl AOsStrExt for OsStr {
    fn strip_prefix(&self, prefix: &str) -> Option<&OsStr> {
        strip_prefix(self, prefix)
    }

    fn split_once(&self, ch: char) -> Option<(&OsStr, &OsStr)> {
        split_once(self, ch)
    }
}

/// Parse the input command line item with given regexs, return an [`CLOpt`].
///
/// The struct of the input option string are:
///
/// ```!
/// [--][/][option][=][value]
///   |  |     |    |    |
///   |  |     |    |    |
///   |  |     |    |    The value part, it is optional.
///   |  |     |    |
///   |  |     |    The delimiter of option name and value.
///   |  |     |    
///   |  |     The option name part, it must be provide by user.
///   |  |
///   |  The disable symbol, generally it is using for boolean option.
///   |  
///   The prefix of option.
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
///     let prefixs = vec![astr("--"), astr("-")];
///
///     {// parse option with value
///         let output = RawVal::from("--foo=32").parse_arg(&prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("foo")));
///         assert_eq!(output.value, Some(Arc::new(RawVal::from("32"))));
///         assert_eq!(output.disable, false);
///     }
///     {// parse boolean option
///         let output = RawVal::from("--/bar").parse_arg(&prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("bar")));
///         assert_eq!(output.value, None);
///         assert_eq!(output.disable, true);
///     }
///     {// parse other string
///         let output = RawVal::from("-=bar").parse_arg(&prefixs);
///
///         assert!(output.is_err());
///     }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CLOpt {
    pub name: Option<Str>,

    pub value: Option<Arc<RawVal>>,

    pub prefix: Option<Str>,

    pub disable: bool,
}

impl CLOpt {
    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn value(&self) -> Option<&Arc<RawVal>> {
        self.value.as_ref()
    }

    pub fn prefix(&self) -> Option<&Str> {
        self.prefix.as_ref()
    }

    pub fn disable(&self) -> bool {
        self.disable
    }
}

const EQUAL: char = '=';

const DISBALE: &'static str = "/";

#[cfg(not(feature = "utf8"))]
impl ArgParser for RawVal {
    type Output = CLOpt;

    type Error = Error;

    fn parse_arg(&self, prefixs: &[Str]) -> Result<Self::Output, Self::Error> {
        for prefix in prefixs {
            if let Some(with_out_pre) = self.strip_prefix(prefix.as_str()) {
                let (dsb, left) = if let Some(left) = with_out_pre.strip_prefix(DISBALE) {
                    (true, left)
                } else {
                    (false, with_out_pre)
                };
                let (name, value) = if let Some((name, value)) = left.split_once(EQUAL) {
                    (name, Some(value))
                } else {
                    (left, None)
                };
                let name = name
                    .to_str()
                    .ok_or_else(|| Error::arg_missing_name(format!("Name must be valid utf8")))?
                    .trim();

                if name.is_empty() {
                    return Err(Error::arg_missing_name(format!("Name can not be empty")));
                }
                return Ok(Self::Output {
                    disable: dsb,
                    name: Some(astr(name)),
                    value: value.map(|v| Arc::new(v.to_os_string().into())),
                    prefix: Some(prefix.clone()),
                });
            }
        }
        Err(Error::arg_parsing_failed(format!(
            "Not a valid option setting string: {:?}",
            self
        )))
    }
}

#[cfg(feature = "utf8")]
impl ArgParser for RawVal {
    type Output = CLOpt;

    type Error = Error;

    fn parse_arg(&self, prefixs: &[Str]) -> Result<Self::Output, Self::Error> {
        for prefix in prefixs {
            if let Some(with_out_pre) = self.strip_prefix(prefix.as_str()) {
                let (dsb, left) = if let Some(left) = with_out_pre.strip_prefix(DISBALE) {
                    (true, left)
                } else {
                    (false, with_out_pre)
                };
                let (name, value) = if let Some((name, value)) = left.split_once(EQUAL) {
                    (name, Some(value))
                } else {
                    (left, None)
                };
                let name = name
                    .trim();

                if name.is_empty() {
                    return Err(Error::arg_missing_name(format!("Name can not be empty")));
                }
                return Ok(Self::Output {
                    disable: dsb,
                    name: Some(astr(name)),
                    value: value.map(|v| Arc::new(v.into())),
                    prefix: Some(prefix.clone()),
                });
            }
        }
        Err(Error::arg_parsing_failed(format!(
            "Not a valid option setting string: {:?}",
            self
        )))
    }
}
