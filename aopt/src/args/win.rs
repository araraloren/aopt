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

const DISBALE: &str = "/";

#[cfg(not(feature = "utf8"))]
impl ArgParser for RawVal {
    type Output = CLOpt;

    type Error = Error;

    fn parse_arg(&self, prefixs: &[Str]) -> Result<Self::Output, Self::Error> {
        for prefix in prefixs {
            // - remove the prefix from the string
            if let Some(with_out_pre) = self.strip_prefix(prefix.as_str()) {
                // - split the string once by delimiter `DISABLE`
                // - split the string once by delimiter `EQUAL`
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

#[cfg(feature = "utf8")]
impl ArgParser for RawVal {
    type Output = CLOpt;

    type Error = Error;

    fn parse_arg(&self, prefixs: &[Str]) -> Result<Self::Output, Self::Error> {
        use std::ops::Deref;

        for prefix in prefixs {
            let inner = self.deref();

            if let Some(with_out_pre) = inner.strip_prefix(prefix.as_str()) {
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
                let name = name.trim();

                if name.is_empty() {
                    return Err(Error::arg_missing_name("Name can not be empty"));
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
