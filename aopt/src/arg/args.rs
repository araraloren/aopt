use std::ffi::OsString;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug, Clone, Default)]
pub struct Args {
    inner: Vec<OsString>,
}

impl Args {
    pub fn new<S: Into<OsString>>(inner: impl Iterator<Item = S>) -> Self {
        Self {
            inner: inner.map(|v| v.into()).collect(),
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.inner, self.len())
    }
}

impl Deref for Args {
    type Target = Vec<OsString>;

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
    inner: &'a [OsString],
    index: usize,
    total: usize,
}

impl<'a> Iter<'a> {
    pub fn new(iter: &'a [OsString], total: usize) -> Self {
        Self {
            inner: iter,
            index: 0,
            total,
        }
    }

    pub fn len(&self) -> usize {
        self.total
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a OsString, Option<&'a OsString>);

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
        self.total
    }
}

#[cfg(test)]
mod test {

    use super::Args;
    use std::ffi::OsString;

    #[test]
    fn test_args() {
        let mut args = Args::new(["--opt", "value", "--bool", "pos"].into_iter());
        let mut iter = args.iter().enumerate();

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 0);
            assert_eq!(opt, &OsString::from("--opt"));
            assert_eq!(arg, Some(&OsString::from("value")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 1);
            assert_eq!(opt, &OsString::from("value"));
            assert_eq!(arg, Some(&OsString::from("--bool")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 2);
            assert_eq!(opt, &OsString::from("--bool"));
            assert_eq!(arg, Some(&OsString::from("pos")));
        }

        if let Some((idx, (opt, arg))) = iter.next() {
            assert_eq!(idx, 3);
            assert_eq!(opt, &OsString::from("pos"));
            assert_eq!(arg, None);
        }

        assert_eq!(iter.next(), None);
    }
}
