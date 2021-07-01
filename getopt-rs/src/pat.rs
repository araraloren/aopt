use std::iter::Skip;
use std::str::Chars;

/// Pattern holder of the user input command line
/// and create info string of option.
#[derive(Debug)]
pub struct ParserPattern<'pat, 'pre> {
    pattern: &'pat str,

    support_prefix: &'pre[String],
}

impl<'pat, 'pre> ParserPattern<'pat, 'pre> {
    pub fn new(pattern: &'pat str, prefix: &'pre[String]) -> Self {
        Self {
            pattern,
            support_prefix: prefix,
        }
    }

    pub fn get_prefixs(&self) -> &'pre[String] {
        self.support_prefix
    }

    pub fn get_pattern(&self) -> &'pat str {
        self.pattern
    }

    pub fn left_chars(&self, skip_len: usize) -> Skip<Chars> {
        self.pattern.chars().skip(skip_len)
    }

    pub fn len(&self) -> usize {
        self.pattern.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParseIndex(usize, usize);

impl ParseIndex {
    pub fn new(len: usize) -> Self {
        Self(0, len)
    }

    pub fn get(&self) -> usize {
        self.0
    }

    pub fn is_end(&self) -> bool {
        self.0 == self.1
    }

    pub fn inc(&mut self, len: usize) -> &mut Self {
        self.0 += len;
        self
    }

    pub fn set(&mut self, cur: usize) -> &mut Self {
        self.0 = cur;
        self
    }

    pub fn len(&self) -> usize {
        self.1
    }
}
