use std::ops::Deref;

use super::ArgParser;
use crate::astr;
use crate::Str;

#[derive(Debug, Clone)]
pub struct Args(Vec<Str>, usize);

impl Args {
    pub fn new<I, ITER>(iter: ITER) -> Self
    where
        I: Into<Str>,
        ITER: Iterator<Item = I>,
    {
        let iter = iter.map(|v| v.into());

        Self(iter.collect(), 0)
    }

    pub fn inner(&self) -> &Vec<Str> {
        &self.0
    }

    pub fn inner_iter(&self) -> std::slice::Iter<'_, Str> {
        self.0.iter()
    }

    /// Increment the argument index.
    pub fn skip(&mut self) {
        self.1 += 1;
    }

    pub fn get_index(&self) -> usize {
        self.1
    }

    pub fn get_curr(&self) -> Option<&Str> {
        self.0.get(self.get_index())
    }

    pub fn get_next(&self) -> Option<&Str> {
        self.0.get(self.get_index() + 1)
    }

    pub fn is_last(&self) -> bool {
        self.1 >= self.0.len()
    }

    /// Parsing current command line argument item with given [`ArgParser`] `P`.
    pub fn parse<P: ArgParser>(
        &self,
        parser: &mut P,
        prefixs: &[Str],
    ) -> Result<P::Output, P::Error> {
        parser.parse(self.0[self.1].clone(), prefixs)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<Str>> for Args {
    fn from(v: Vec<Str>) -> Self {
        Args(v, 0)
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

impl From<std::env::Args> for Args {
    fn from(v: std::env::Args) -> Self {
        Args(
            v.collect::<Vec<String>>()
                .iter()
                .map(|v| astr(v.as_str()))
                .collect(),
            0,
        )
    }
}

/// The default value of [`Args`] are `std::env::args()[1..]`.
impl Default for Args {
    fn default() -> Self {
        Self(
            std::env::args().skip(1).map(|v| astr(v.as_str())).collect(),
            0,
        )
    }
}

#[cfg(test)]
mod test {

    use super::Args;
    use crate::astr;

    #[test]
    fn test_args() {
        let mut args = Args::new(["--opt", "value", "--bool", "pos"].into_iter());

        assert_eq!(args.get_index(), 0);
        assert_eq!(args.get_curr(), Some(&astr("--opt")));
        assert_eq!(args.get_next(), Some(&astr("value")));

        args.skip();

        assert_eq!(args.get_index(), 1);
        assert_eq!(args.get_curr(), Some(&astr("value")));
        assert_eq!(args.get_next(), Some(&astr("--bool")));

        args.skip();

        assert_eq!(args.get_index(), 2);
        assert_eq!(args.get_curr(), Some(&astr("--bool")));
        assert_eq!(args.get_next(), Some(&astr("pos")));

        args.skip();

        assert_eq!(args.get_index(), 3);
        assert_eq!(args.get_curr(), Some(&astr("pos")));
        assert_eq!(args.get_next(), None);

        args.skip();

        assert!(args.is_last());
        assert_eq!(args.get_index(), 4);
        assert_eq!(args.get_curr(), None);
        assert_eq!(args.get_next(), None);
    }
}
