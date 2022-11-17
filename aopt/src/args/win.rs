use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use super::ArgParser;
use crate::astr;
use crate::Arc;
use crate::Error;
use crate::Str;

fn strip_prefix(str: &OsStr, prefix: &str) -> Option<OsString> {
    let enc = str.encode_wide();
    let mut pre = prefix.encode_utf16();
    let mut ret = Vec::with_capacity(str.len().saturating_sub(prefix.len()));

    for ori in enc {
        match pre.next() {
            Some(ch) => {
                if ch != ori {
                    return None;
                }
            }
            None => {
                ret.push(ori);
            }
        }
    }
    Some(OsString::from_wide(&ret))
}

fn split_once(str: &OsStr, ch: char) -> Option<(OsString, OsString)> {
    let enc = str.encode_wide();
    let mut buf = [0; 1];
    let sep = ch.encode_utf16(&mut buf);
    let enc = enc.collect::<Vec<u16>>();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .and_then(|(idx, _)| {
            Some((
                OsString::from_wide(&enc[0..idx]),
                OsString::from_wide(&enc[idx + 1..]),
            ))
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
/// # use std::ffi::OsStr;
/// # use std::ffi::OsString;
/// # use aopt::args::ArgParser;
/// #
/// # fn main() -> Result<(), Error> {
///     let prefixs = vec![astr("--"), astr("-")];
///
///     {// parse option with value
///         let output = OsStr::new("--foo=32").parse(&prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("foo")));
///         assert_eq!(output.value, Some(Arc::new(OsString::from("32"))));
///         assert_eq!(output.disable, false);
///     }
///     {// parse boolean option
///         let output = OsStr::new("--/bar").parse(&prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("bar")));
///         assert_eq!(output.value, None);
///         assert_eq!(output.disable, true);
///     }
///     {// parse other string
///         let output = OsStr::new("-=bar").parse(&prefixs);
///
///         assert!(output.is_err());
///     }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CLOpt {
    pub name: Option<Str>,

    pub value: Option<Arc<OsString>>,

    pub prefix: Option<Str>,

    pub disable: bool,
}

impl CLOpt {
    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn value(&self) -> Option<&Arc<OsString>> {
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

impl ArgParser for OsStr {
    type Output = CLOpt;

    type Error = Error;

    fn parse(&self, prefixs: &[Str]) -> Result<Self::Output, Self::Error> {
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
                    .ok_or_else(|| {
                        Error::arg_missing_name(format!("Name must be valid utf8: {:?}", name))
                    })?
                    .trim();

                if name.is_empty() {
                    return Err(Error::arg_missing_name("Name can not be empty"));
                }
                return Ok(Self::Output {
                    disable: dsb,
                    name: Some(astr(name)),
                    value: value.map(|v| v.into()),
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
