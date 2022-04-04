use ustr::Ustr;

/// Pattern holder of the user input command line
/// and create info string of option.
#[derive(Debug)]
pub struct ParserPattern<'pre> {
    pattern: Ustr,

    pattern_chars: Vec<char>,

    support_prefix: &'pre [Ustr],
}

impl<'pre> ParserPattern<'pre> {
    pub fn new(pattern: Ustr, support_prefix: &'pre [Ustr]) -> Self {
        Self {
            pattern,
            pattern_chars: pattern.chars().collect(),
            support_prefix,
        }
    }

    pub fn get_prefix(&self) -> Option<&Ustr> {
        self.support_prefix
            .iter()
            .find(|v| self.pattern.starts_with(v.as_ref()))
    }

    pub fn get_pattern(&self) -> &str {
        self.pattern.as_ref()
    }

    pub fn get_chars(&self, offset: usize) -> &[char] {
        &self.pattern_chars[offset..]
    }

    pub fn get_subchars(&self, from: usize, end: usize) -> &[char] {
        &self.pattern_chars[from..end]
    }

    pub fn get_substr(&self, from: usize, end: usize) -> Ustr {
        crate::gstr(&self.pattern_chars[from..end].iter().fold(
            String::with_capacity(end - from + 1),
            |mut a, v| {
                a.push(*v);
                a
            },
        ))
    }

    pub fn starts(&self, ch: char, skip_len: usize) -> bool {
        self.pattern_chars.get(skip_len) == Some(&ch)
    }

    pub fn len(&self) -> usize {
        self.pattern_chars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
