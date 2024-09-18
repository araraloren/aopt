use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::ops::Deref;

use crate::parser::ReturnVal;
use crate::ARef;

#[derive(Debug, Clone, Default)]
pub struct Args<'a> {
    inner: ARef<Vec<Cow<'a, OsStr>>>,
}

impl<'a> Args<'a> {
    pub fn new<S: IntoArg<'a>>(inner: impl Iterator<Item = S>) -> Self {
        Self {
            inner: ARef::new(inner.map(|v| v.into_arg()).collect()),
        }
    }

    /// Create from [`args_os`](std::env::args_os()).
    pub fn from_env() -> Self {
        Self::new(std::env::args_os())
    }

    pub fn iter2(&self) -> impl Iterator<Item = (&Cow<'a, OsStr>, Option<&Cow<'a, OsStr>>)> {
        self.inner
            .iter()
            .zip(self.inner.iter().skip(1).map(|v| Some(v)).chain(None))
    }

    pub fn unwrap_or_clone(self) -> Vec<Cow<'a, OsStr>> {
        ARef::unwrap_or_clone(self.inner)
    }
}

impl<'a, T: IntoArg<'a>, I: IntoIterator<Item = T>> From<I> for Args<'a> {
    fn from(value: I) -> Self {
        Self::new(value.into_iter())
    }
}

impl<'a> From<Args<'a>> for Vec<Cow<'a, OsStr>> {
    fn from(value: Args<'a>) -> Self {
        value.unwrap_or_clone()
    }
}

impl<'a> From<ReturnVal<'a>> for Args<'a> {
    fn from(mut value: ReturnVal<'a>) -> Self {
        value.take_ctx().take_args()
    }
}

impl<'a> From<&ReturnVal<'a>> for Args<'a> {
    fn from(value: &ReturnVal<'a>) -> Self {
        value.ctx().args().clone()
    }
}

impl<'a> From<&mut ReturnVal<'a>> for Args<'a> {
    fn from(value: &mut ReturnVal<'a>) -> Self {
        value.take_ctx().take_args()
    }
}

impl<'a> Deref for Args<'a> {
    type Target = Vec<Cow<'a, OsStr>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for Args<'_> {
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

pub trait IntoArg<'a> {
    fn into_arg(self) -> Cow<'a, OsStr>;
}

impl<'a> IntoArg<'a> for &'a str {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<'a> IntoArg<'a> for String {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Owned(OsString::from(self))
    }
}

impl<'a> IntoArg<'a> for &'a String {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<'a> IntoArg<'a> for &'a mut String {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Borrowed(AsRef::as_ref(self))
    }
}

impl<'a> IntoArg<'a> for &'a OsStr {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Borrowed(self)
    }
}

impl<'a> IntoArg<'a> for OsString {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Owned(self)
    }
}

impl<'a> IntoArg<'a> for &'a OsString {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<'a> IntoArg<'a> for &'a mut OsString {
    fn into_arg(self) -> Cow<'a, OsStr> {
        Cow::Borrowed(AsRef::as_ref(self))
    }
}

#[cfg(test)]
mod test {

    use std::ffi::OsStr;

    use super::Args;

    #[test]
    fn test_args() {
        let args = Args::from(["--opt", "value", "--bool", "pos"]);
        let mut iter = args.iter2().enumerate();

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
