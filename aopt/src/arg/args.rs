use std::ops::Deref;
use std::ops::DerefMut;

use super::ArgParser;
use crate::astr;
use crate::Str;

#[derive(Debug, Clone)]
pub struct Args(Vec<Str>);

impl Args {
    pub fn new<I, ITER>(iter: ITER) -> Self
    where
        I: Into<Str>,
        ITER: Iterator<Item = I>,
    {
        let iter = iter.map(|v| v.into());

        Self(iter.collect())
    }

    /// Collect arguments using [`args`](std::env::args), and skip first argument.
    pub fn new_args() -> Self {
        Self(std::env::args().skip(1).map(|v| astr(v)).collect())
    }

    pub fn iter(&self) -> ArgsIter<std::slice::Iter<Str>> {
        ArgsIter::new(self.0.iter(), 0, self.len())
    }

    pub fn into_iter(self) -> ArgsIter<std::vec::IntoIter<Str>> {
        let len = self.len();
        ArgsIter::new(self.0.into_iter(), 0, len)
    }
}

impl From<Vec<Str>> for Args {
    fn from(v: Vec<Str>) -> Self {
        Args(v)
    }
}

impl<'a> From<&'a [Str]> for Args {
    fn from(v: &'a [Str]) -> Self {
        Args::from(v.to_vec())
    }
}

impl Deref for Args {
    type Target = Vec<Str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Args {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Args {
    fn default() -> Self {
        Self(Vec::default())
    }
}

#[derive(Debug, Clone)]
pub struct ArgsIter<I> {
    iter: I,
    index: usize,
    total: usize,
    args: (Option<Str>, Option<Str>),
}

impl<T, I> ArgsIter<I>
where
    I: Iterator<Item = T> + Clone,
    T: Into<Str>,
{
    pub fn new(mut iter: I, index: usize, total: usize) -> Self {
        let first = iter.next().map(|v| v.into());
        Self {
            iter,
            index,
            total,
            args: (None, first),
        }
    }

    pub fn cur(&self) -> Option<&Str> {
        self.args.0.as_ref()
    }

    pub fn arg(&self) -> Option<&Str> {
        self.args.1.as_ref()
    }

    pub fn idx(&self) -> usize {
        self.index - 1
    }

    pub fn is_last(&self) -> bool {
        self.index > self.total
    }

    /// Parsing current command line argument item with given [`ArgParser`] `P`.
    pub fn parse<P: ArgParser>(
        &self,
        parser: &mut P,
        prefixs: &[Str],
    ) -> Result<P::Output, P::Error> {
        let pattern = self.args.0.clone();
        parser.parse(pattern.unwrap(), prefixs)
    }

    pub fn len(&self) -> usize {
        self.total
    }
}

impl<T, I> Iterator for ArgsIter<I>
where
    I: Iterator<Item = T> + Clone,
    T: Into<Str>,
{
    type Item = Str;

    fn next(&mut self) -> Option<Self::Item> {
        self.args.0 = self.args.1.take();
        self.args.1 = self.iter.next().map(|v| v.into());
        self.index += 1;
        self.args.0.clone()
    }
}

impl<T, I> ExactSizeIterator for ArgsIter<I>
where
    T: Into<Str>,
    I: Iterator<Item = T> + Clone + ExactSizeIterator,
{
    fn len(&self) -> usize {
        let (lower, upper) = self.iter.size_hint();
        // Note: This assertion is overly defensive, but it checks the invariant
        // guaranteed by the trait. If this trait were rust-internal,
        // we could use debug_assert!; assert_eq! will check all Rust user
        // implementations too.
        assert_eq!(upper, Some(lower));
        lower
    }
}

#[cfg(test)]
mod test {

    use super::Args;
    use crate::astr;

    #[test]
    fn test_args() {
        let args = Args::new(["--opt", "value", "--bool", "pos"].into_iter());
        let mut iter = args.iter();

        iter.next();

        assert_eq!(iter.idx(), 0);
        assert_eq!(iter.cur(), Some(&astr("--opt")));
        assert_eq!(iter.arg(), Some(&astr("value")));

        iter.next();

        assert_eq!(iter.idx(), 1);
        assert_eq!(iter.cur(), Some(&astr("value")));
        assert_eq!(iter.arg(), Some(&astr("--bool")));

        iter.next();

        assert_eq!(iter.idx(), 2);
        assert_eq!(iter.cur(), Some(&astr("--bool")));
        assert_eq!(iter.arg(), Some(&astr("pos")));

        iter.next();

        assert_eq!(iter.idx(), 3);
        assert_eq!(iter.cur(), Some(&astr("pos")));
        assert_eq!(iter.arg(), None);

        iter.next();

        assert!(iter.is_last());
        assert_eq!(iter.idx(), 4);
        assert_eq!(iter.cur(), None);
        assert_eq!(iter.arg(), None);
    }
}
