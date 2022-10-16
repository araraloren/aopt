use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

fn strip_pre<'a>(str: &'a OsStr, prefix: &str) -> Option<&'a OsStr> {
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
    fn strip_pre(&self, prefix: &str) -> Option<&OsStr>;

    fn split_once(&self, ch: char) -> Option<(&OsStr, &OsStr)>;
}

impl AOsStrExt for OsStr {
    fn strip_pre(&self, prefix: &str) -> Option<&OsStr> {
        strip_pre(self, prefix)
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
/// extern crate aopt as test_crate;
///
/// use test_crate::arg::CLOptParser;
/// use test_crate::arg::ArgParser;
/// use test_crate::astr;
/// use test_crate::err::Result;
///
/// fn main() -> Result<()> {
///     let mut parser = CLOptParser::default();
///     let prefixs = vec![astr("--"), astr("-")];
///
///     {// parse option with value
///         let output = parser.parse(astr("--foo=32"), &prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("foo")));
///         assert_eq!(output.value, Some(astr("32")));
///         assert_eq!(output.disable, false);
///     }
///     {// parse boolean option
///         let output = parser.parse(astr("--/bar"), &prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("bar")));
///         assert_eq!(output.value, None);
///         assert_eq!(output.disable, true);
///     }
///     {// parse other string
///         let output = parser.parse(astr("-=bar"), &prefixs);
///
///         assert!(output.is_err());
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CLOpt {
    pub name: Option<Str>,

    pub value: Option<OsString>,

    pub prefix: Option<Str>,

    pub disable: bool,
}

impl CLOpt {
    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn val(&self) -> Option<&OsStr> {
        self.value.map(|v| v.as_ref())
    }

    pub fn pre(&self) -> Option<&Str> {
        self.prefix.as_ref()
    }

    pub fn dsb(&self) -> bool {
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
            if let Some(with_out_pre) = self.strip_pre(prefix.as_str()) {
                let (dsb, left) = if let Some(left) = with_out_pre.strip_pre(Self::DISBALE) {
                    (true, left)
                } else {
                    (false, with_out_pre)
                };
                let (name, value) = if let Some((name, value)) = left.split_once(Self::EQUAL) {
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
                    value: value.map(|v| v.into()),
                    prefix: Some(prefix.clone()),
                });
            }
        }
        Err(Error::arg_parsing_failed(""))
    }
}
