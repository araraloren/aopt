#[cfg_attr(windows, path = "args/win.rs")]
#[cfg_attr(not(windows), path = "args/unix.rs")]
pub(crate) mod osstr_ext;

use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::parser::ReturnVal;
use crate::Error;
use crate::RawVal;

pub use self::osstr_ext::split_once;
pub use self::osstr_ext::strip_prefix;
pub use self::osstr_ext::CLOpt;

pub trait ArgParser {
    type Output;
    type Error: Into<Error>;

    fn parse_arg(&self) -> Result<Self::Output, Self::Error>;
}
#[derive(Debug, Clone, Default)]
pub struct Args {
    inner: Vec<RawVal>,
}

impl Args {
    pub fn new<S: Into<RawVal>>(inner: impl Iterator<Item = S>) -> Self {
        Self {
            inner: inner.map(|v| v.into()).collect(),
        }
    }

    /// Create from [`args_os`](std::env::args_os()).
    pub fn from_env() -> Self {
        Self::new(std::env::args_os())
    }

    pub fn guess_iter(&self) -> Iter<'_> {
        Iter::new(&self.inner)
    }
}

impl<T: Into<RawVal>, I: IntoIterator<Item = T>> From<I> for Args {
    fn from(value: I) -> Self {
        Self::new(value.into_iter())
    }
}

impl From<Args> for Vec<RawVal> {
    fn from(value: Args) -> Self {
        value.inner
    }
}

impl From<ReturnVal> for Args {
    fn from(value: ReturnVal) -> Self {
        Self::new(value.clone_args().into_iter())
    }
}

impl<'a> From<&'a ReturnVal> for Args {
    fn from(value: &'a ReturnVal) -> Self {
        Self::new(value.args().iter().cloned())
    }
}

impl<'a> From<&'a mut ReturnVal> for Args {
    fn from(value: &'a mut ReturnVal) -> Self {
        Self::new(value.clone_args().into_iter())
    }
}

impl Deref for Args {
    type Target = Vec<RawVal>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Args {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
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

#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: &'a [RawVal],
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new(iter: &'a [RawVal]) -> Self {
        Self {
            inner: iter,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a RawVal, Option<&'a RawVal>);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;

        if index < self.len() {
            let may_opt = &self.inner[index];
            let may_arg = (index + 1 < self.len()).then(|| &self.inner[index + 1]);

            self.index += 1;
            Some((may_opt, may_arg))
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(test)]
mod test {

    use super::Args;
    use crate::RawVal;

    #[test]
    fn test_args() {
        let args = Args::from(["--opt", "value", "--bool", "pos"]);
        let mut iter = args.guess_iter().enumerate();

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 0);
            assert_eq!(opt, &RawVal::from("--opt"));
            assert_eq!(arg, Some(&RawVal::from("value")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 1);
            assert_eq!(opt, &RawVal::from("value"));
            assert_eq!(arg, Some(&RawVal::from("--bool")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 2);
            assert_eq!(opt, &RawVal::from("--bool"));
            assert_eq!(arg, Some(&RawVal::from("pos")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 3);
            assert_eq!(opt, &RawVal::from("pos"));
            assert_eq!(arg, None);
        }

        assert_eq!(iter.next(), None);
    }
}
