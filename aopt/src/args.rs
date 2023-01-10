#[cfg_attr(windows, path = "args/win.rs")]
#[cfg_attr(unix, path = "args/unix.rs")]
pub(crate) mod osstr_ext;

use std::ops::Deref;
use std::ops::DerefMut;

use crate::parser::ReturnVal;
use crate::Error;
use crate::RawVal;

pub use self::osstr_ext::AOsStrExt;
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

    cfg_if::cfg_if! {
        if #[cfg(feature = "utf8")] {
            /// Create from [`args_os`](std::env::args_os()).
            pub fn from_env() -> Self {
                Self::new(std::env::args_os())
            }
        }
        else {
            /// Create from [`args`](std::env::args()).
            pub fn from_env() -> Self {
                Self::new(std::env::args())
            }
        }
    }

    pub fn guess_iter(&self) -> Iter<'_> {
        Iter::new(&self.inner)
    }

    pub fn into_inner(self) -> Vec<RawVal> {
        self.inner
    }
}

impl<S: Into<RawVal>, I: Iterator<Item = S>> From<I> for Args {
    fn from(iter: I) -> Self {
        Self::new(iter)
    }
}

impl From<ReturnVal> for Args {
    fn from(value: ReturnVal) -> Self {
        Self::from(value.clone_args().into_iter())
    }
}

impl<'a> From<&'a ReturnVal> for Args {
    fn from(value: &'a ReturnVal) -> Self {
        Self::from(value.args().iter().cloned())
    }
}

impl<'a> From<&'a mut ReturnVal> for Args {
    fn from(value: &'a mut ReturnVal) -> Self {
        Self::from(value.clone_args().into_iter())
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
        if self.index < self.len() {
            let index = self.index;

            self.index += 1;
            Some((&self.inner[index], self.inner.get(index + 1)))
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
        let args = Args::new(["--opt", "value", "--bool", "pos"].into_iter());
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
