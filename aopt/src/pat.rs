use std::iter::Skip;
use std::str::Chars;

use ustr::Ustr;

/// Pattern holder of the user input command line
/// and create info string of option.
#[derive(Debug)]
pub struct ParserPattern<'pre> {
    pattern: Ustr,

    support_prefix: &'pre [Ustr],
}

impl<'pre> ParserPattern<'pre> {
    pub fn new(pattern: Ustr, prefix: &'pre [Ustr]) -> Self {
        Self {
            pattern,
            support_prefix: prefix,
        }
    }

    pub fn get_prefixs(&self) -> &'pre [Ustr] {
        self.support_prefix
    }

    pub fn get_pattern(&self) -> &str {
        self.pattern.as_ref()
    }

    pub fn chars(&self, skip_len: usize) -> Skip<Chars> {
        self.pattern.chars().skip(skip_len)
    }

    pub fn starts(&self, ch: char, skip_len: usize) -> bool {
        self.pattern.chars().nth(skip_len + 1) == Some(ch)
    }

    pub fn len(&self) -> usize {
        self.pattern.len()
    }

    pub fn clone_pattern(&self) -> Ustr {
        self.pattern
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
