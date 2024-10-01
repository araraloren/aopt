use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::ops::Deref;

use crate::parser::Return;
use crate::str::CowOsStrUtils;
use crate::ARef;
use crate::Error;

const EQUAL: char = '=';

#[derive(Debug, Clone, Default)]
pub struct ArgInfo<'a> {
    pub name: Cow<'a, str>,

    pub value: Option<Cow<'a, OsStr>>,
}

impl<'a> ArgInfo<'a> {
    pub fn parse(val: &'a OsStr) -> Result<Self, Error> {
        let arg_display = format!("{}", std::path::Path::new(val).display());

        crate::trace!("parsing command line argument {val:?}");
        if let Some((name, value)) = crate::str::split_once(val, EQUAL) {
            // - convert the name to &str, the name must be valid utf8
            let name = name
                .to_str(|v| v.trim())
                .ok_or_else(|| Error::arg(&arg_display, "failed convert OsStr to str"))?;

            if name.is_empty() {
                return Err(Error::arg(arg_display, "can not be empty"));
            }
            Ok(Self {
                name,
                value: Some(value),
            })
        } else {
            let name = val
                .to_str()
                .ok_or_else(|| Error::arg(arg_display, "failed convert OsStr to str"))?;

            Ok(Self {
                name: Cow::Borrowed(name),
                value: None,
            })
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Args {
    inner: ARef<Vec<OsString>>,
}

impl Args {
    pub fn new<S: Into<OsString>>(inner: impl Iterator<Item = S>) -> Self {
        Self {
            inner: ARef::new(inner.map(|v| v.into()).collect()),
        }
    }

    /// Create from [`args_os`](std::env::args_os()).
    pub fn from_env() -> Self {
        Self::new(std::env::args_os())
    }

    pub fn unwrap_or_clone(self) -> Vec<OsString> {
        ARef::unwrap_or_clone(self.inner)
    }
}

impl<T: Into<OsString>, I: IntoIterator<Item = T>> From<I> for Args {
    fn from(value: I) -> Self {
        Self::new(value.into_iter())
    }
}

impl From<Args> for Vec<OsString> {
    fn from(value: Args) -> Self {
        value.unwrap_or_clone()
    }
}

impl From<Return> for Args {
    fn from(mut value: Return) -> Self {
        Self::new(value.take_args().into_iter())
    }
}

impl From<&Return> for Args {
    fn from(value: &Return) -> Self {
        Self::new(value.clone_args().into_iter())
    }
}

impl From<&mut Return> for Args {
    fn from(value: &mut Return) -> Self {
        Self::new(value.take_args().into_iter())
    }
}

impl Deref for Args {
    type Target = Vec<OsString>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub fn iter2<'a, 'b>(
    args: &'a [&'b OsStr],
) -> impl Iterator<Item = (&'a &'b OsStr, Option<&'a &'b OsStr>)> {
    args.iter()
        .scan(args.iter().skip(1), |i, e| Some((e, i.next())))
}

impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Args {{[{}]}}",
            self.inner
                .iter()
                .map(|v| format!("{:?}", v))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod test {

    use std::ffi::OsStr;

    use super::Args;

    #[test]
    fn test_args() {
        let args = Args::from(["--opt", "value", "--bool", "pos"]);
        let mut iter = args
            .iter()
            .zip(args.iter().skip(1).map(Some).chain(None))
            .enumerate();

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 0);
            assert_eq!(opt, OsStr::new("--opt"));
            assert_eq!(arg.map(|v| v.as_ref()), Some(OsStr::new("value")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 1);
            assert_eq!(opt, OsStr::new("value"));
            assert_eq!(arg.map(|v| v.as_ref()), Some(OsStr::new("--bool")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 2);
            assert_eq!(opt, OsStr::new("--bool"));
            assert_eq!(arg.map(|v| v.as_ref()), Some(OsStr::new("pos")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 3);
            assert_eq!(opt, OsStr::new("pos"));
            assert_eq!(arg, None);
        }

        assert_eq!(iter.next(), None);
    }
}
