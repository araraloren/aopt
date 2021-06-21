

use std::borrow::Cow;
use std::str::Chars;
use std::iter::Skip;

use crate::str::Str;

#[derive(Debug)]
pub struct ParserPattern<'a, 'b, 'c> {
    pattern: &'a str,

    support_prefix: &'b Vec<Str<'c>>,

    current: usize,
}

impl<'a, 'b, 'c> ParserPattern<'a, 'b, 'c> {
    pub fn new(pattern: &'a str, prefix: &'b Vec<Str<'c>>) -> Self {
        Self {
            pattern,
            support_prefix: prefix,
            current: 0,
        }
    }

    pub fn get_pattern(&self) -> &'a str {
        self.pattern
    }

    pub fn is_end(&self) -> bool {
        self.current == self.pattern.len()
    }

    pub fn inc_current(&mut self, len: usize) -> &mut Self {
        self.current += len;
        self
    }

    pub fn set_current(&mut self, cur: usize) -> &mut Self {
        self.current = cur;
        self
    }

    pub fn left_chars(&self) -> Skip<Chars> {
        self.pattern.chars().skip(self.current)
    }

    pub fn len(&self) -> usize {
        self.pattern.len()
    }
}