use ustr::Ustr;

use super::index::Index;

use crate::err::Error;
use crate::err::Result;
use crate::pat::ParseIndex;
use crate::pat::ParserPattern;

/// Parse the option string with given prefixs, return an [`DataKeeper`].
///
/// The struct of the option string are:
///
/// ```!
/// [--][option][=][type][/][!][@index]
///  |     |     |    |   |  |   |
///  |     |     |    |   |  |   |
///  |     |     |    |   |  |   |
///  |     |     |    |   |  |   The index part of option. Here are all the possible string:
///  |     |     |    |   |  |   @1 means first position
///  |     |     |    |   |  |   @-1 means last position
///  |     |     |    |   |  |   @[1, 2, 3] means the position 1, 2 and 3
///  |     |     |    |   |  |   @-[1, 2] means except the position 1, 2
///  |     |     |    |   |  |   @>2 means position that bigger than 2
///  |     |     |    |   |  |   @<3 means position less than 3
///  |     |     |    |   |  |   @* means all the position
///  |     |     |    |   |  |
///  |     |     |    |   |  Indicate the option is force required.
///  |     |     |    |   |
///  |     |     |    |   The disable symbol, generally it is using for boolean option.
///  |     |     |    |
///  |     |     |    The type name of option.
///  |     |     |    
///  |     |     The delimiter of option name and type.
///  |     |
///  |     The option name part, it must be provide by user.
///  |  
///  The prefix of option.
/// ```
///
/// # Example
///
/// ```rust
/// use aopt::gstr;
/// use aopt::err::Result;
/// use aopt::opt::parse_option_str;
///
/// fn main() -> Result<()> {
///     let ret = parse_option_str("--aopt=t!/".into(), &[gstr("--")])?;
///
///     assert_eq!(ret.prefix, Some(gstr("--")));
///     assert_eq!(ret.name , Some(gstr("aopt")));
///     assert_eq!(ret.type_name, Some(gstr("t")));
///     assert_eq!(ret.deactivate, Some(true));
///     assert_eq!(ret.optional, Some(true));
///     assert_eq!(ret.forward_index, None);
///     assert_eq!(ret.backward_index, None);
///     assert_eq!(ret.anywhere, None);
///     assert_eq!(ret.list, []);
///     assert_eq!(ret.except, []);
///     assert_eq!(ret.greater, None);
///     assert_eq!(ret.less, None);
///
///     let ret = parse_option_str("bopt=t@[1,2,3]".into(), &[gstr("--")])?;
///
///     assert_eq!(ret.prefix, None);
///     assert_eq!(ret.name , Some(gstr("bopt")));
///     assert_eq!(ret.type_name, Some(gstr("t")));
///     assert_eq!(ret.deactivate, None);
///     assert_eq!(ret.optional, None);
///     assert_eq!(ret.forward_index, None);
///     assert_eq!(ret.backward_index, None);
///     assert_eq!(ret.anywhere, None);
///     assert_eq!(ret.list, [1, 2, 3]);
///     assert_eq!(ret.except, []);
///     assert_eq!(ret.greater, None);
///     assert_eq!(ret.less, None);
///
///     Ok(())
/// }
/// ```
///
/// For more examples, please reference test case [`test_option_str_parser`](../../src/aopt/opt/parser.rs.html#542).
///
pub fn parse_option_str(pattern: Ustr, prefix: &[Ustr]) -> Result<DataKeeper> {
    let pattern = ParserPattern::new(pattern, prefix);
    let mut index = ParseIndex::new(pattern.len());
    let mut data_keeper = DataKeeper::default();

    let res = State::default().parse(&mut index, &pattern, &mut data_keeper)?;

    if res {
        trace!(
            ?pattern,
            ?prefix,
            ?data_keeper,
            "parsing option string successed"
        );
        // don't check anything
        return Ok(data_keeper);
    }
    trace!(
        ?pattern,
        ?prefix,
        ?data_keeper,
        "parsing option string failed"
    );
    Err(Error::opt_parsing_constructor_failed(pattern.get_pattern()))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum State {
    PreCheck,
    Prefix,
    Name,
    Equal,
    Type,
    Deactivate,
    Optional,
    Index,
    FowradIndex,
    BackwardIndex,
    List,
    Except,
    AnyWhere,
    Greater,
    Less,
    End,
}

#[derive(Debug, Default)]
pub struct DataKeeper {
    pub pattern: Ustr,

    pub prefix: Option<Ustr>,

    pub name: Option<Ustr>,

    pub type_name: Option<Ustr>,

    pub deactivate: Option<bool>,

    pub optional: Option<bool>,

    pub forward_index: Option<usize>,

    pub backward_index: Option<usize>,

    pub anywhere: Option<bool>,

    pub list: Vec<usize>,

    pub except: Vec<usize>,

    pub greater: Option<usize>,

    pub less: Option<usize>,
}

impl DataKeeper {
    pub fn gen_index(&mut self) -> Index {
        if self.forward_index.is_some() {
            Index::forward(self.forward_index.unwrap())
        } else if self.backward_index.is_some() {
            Index::backward(self.backward_index.unwrap())
        } else if self.anywhere.unwrap_or(false) {
            Index::anywhere()
        } else if !self.list.is_empty() {
            Index::list(std::mem::take(&mut self.list))
        } else if !self.except.is_empty() {
            Index::except(std::mem::take(&mut self.except))
        } else if self.greater.is_some() {
            Index::greater(self.greater.unwrap())
        } else if self.less.is_some() {
            Index::less(self.less.unwrap())
        } else {
            Index::default()
        }
    }

    pub fn has_index(&mut self) -> bool {
        self.forward_index.is_some()
            || self.backward_index.is_some()
            || self.anywhere.is_some()
            || !self.list.is_empty()
            || !self.except.is_empty()
            || self.greater.is_some()
            || self.less.is_some()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::PreCheck
    }
}

const NAME_VALUE_SPLIT: char = '=';

impl State {
    pub fn anywhere_symbol() -> &'static str {
        "*"
    }

    pub fn self_transition<'pre>(&mut self, index: &ParseIndex, pattern: &ParserPattern<'pre>) {
        let index_not_end = pattern.len() > index.get();
        let next_state = match self.clone() {
            Self::PreCheck => Self::Prefix,
            Self::Prefix => {
                if index_not_end {
                    Self::Name
                } else {
                    Self::End
                }
            }
            Self::Name => {
                if index_not_end {
                    if pattern.starts(NAME_VALUE_SPLIT, index.get()) {
                        Self::Equal
                    } else {
                        Self::Type
                    }
                } else {
                    Self::End
                }
            }
            Self::Equal => Self::Type,
            Self::Type | Self::Deactivate | Self::Optional => {
                if let Some(ch) = pattern.get_chars(index.get()).get(0) {
                    match ch {
                        '!' => Self::Optional,
                        '/' => Self::Deactivate,
                        '@' => Self::Index,
                        _ => Self::End,
                    }
                } else {
                    Self::End
                }
            }
            Self::Index => {
                let (_, index_part) = pattern.get_pattern().split_at(index.get());

                if index_part.starts_with("+[") || index_part.starts_with('[') {
                    Self::List
                } else if index_part.starts_with("-[") {
                    Self::Except
                } else if index_part.starts_with('-') {
                    Self::BackwardIndex
                } else if index_part == Self::anywhere_symbol() {
                    Self::AnyWhere
                } else if index_part.starts_with('>') {
                    Self::Greater
                } else if index_part.starts_with('<') {
                    Self::Less
                } else {
                    Self::FowradIndex
                }
            }
            Self::FowradIndex
            | Self::BackwardIndex
            | Self::List
            | Self::Except
            | Self::AnyWhere
            | Self::Greater
            | Self::Less => Self::End,
            Self::End => {
                unreachable!("The end state can't going on!");
            }
        };
        trace!("transition state from '{:?}' to '{:?}'", self, next_state);
        *self = next_state;
    }

    pub fn parse<'pre>(
        mut self,
        index: &mut ParseIndex,
        pattern: &ParserPattern<'pre>,
        data_keeper: &mut DataKeeper,
    ) -> Result<bool> {
        let current_state = self.clone();

        match current_state {
            Self::PreCheck => {
                if pattern.get_pattern().is_empty() {
                    warn!("got an empty pattern");
                    return Ok(false);
                }
                data_keeper.pattern = pattern.clone_pattern();
            }
            Self::Prefix => {
                if let Some(prefix) = pattern.get_prefix() {
                    data_keeper.prefix = Some(*prefix);
                    index.inc(prefix.chars().count());
                }
            }
            Self::Name => {
                let start = index.get();

                for (cur, ch) in pattern.get_chars(start).iter().enumerate() {
                    let mut name_end = 0;

                    if *ch == '=' || *ch == '!' || *ch == '/' || *ch == '@' {
                        if cur >= 1 {
                            name_end = start + cur;
                        } else if cur == 0 {
                            // current is '='
                            break;
                        }
                    } else if start + cur + 1 == index.len() {
                        name_end = start + cur + 1;
                    }
                    if name_end > 0 {
                        let name = pattern.get_substr(start, name_end);

                        debug!("get name from '{:?}': '{}'", pattern, name);
                        data_keeper.name = Some(name);
                        index.set(name_end);
                        break;
                    }
                }
            }
            Self::Equal => {
                index.inc(1);
            }
            Self::Type => {
                let start = index.get();

                for (cur, ch) in pattern.get_chars(start).iter().enumerate() {
                    let mut type_end = 0;

                    if *ch == '!' || *ch == '/' || *ch == '@' {
                        if cur >= 1 {
                            type_end = start + cur;
                        } else if cur == 0 {
                            // current is '='
                            break;
                        }
                    } else if start + cur + 1 == index.len() {
                        type_end = start + cur + 1;
                    }
                    if type_end > 0 {
                        let type_ = pattern.get_substr(start, type_end);

                        debug!("get type name from {:?}: {}", pattern, type_);
                        data_keeper.type_name = Some(type_);
                        index.set(type_end);
                        break;
                    }
                }
            }
            Self::Deactivate => {
                data_keeper.deactivate = Some(true);
                index.inc(1);
            }
            Self::Optional => {
                data_keeper.optional = Some(true);
                index.inc(1);
            }
            Self::Index => {
                index.inc(1);
            }
            Self::FowradIndex => {
                let index_part = pattern.get_chars(index.get());
                let ret = Self::parse_as_usize(pattern, index_part)?;

                if ret > 0 {
                    data_keeper.forward_index = Some(ret);
                } else {
                    data_keeper.anywhere = Some(true);
                }
                index.set(index.len());
            }
            Self::BackwardIndex => {
                let index_part = pattern.get_chars(index.get() + 1);
                let ret = Self::parse_as_usize(pattern, index_part)?;

                if ret > 0 {
                    data_keeper.backward_index = Some(ret);
                } else {
                    data_keeper.anywhere = Some(true);
                }
                index.set(index.len());
            }
            Self::List => {
                let index_part = pattern.get_chars(index.get());
                let mut start_index = index.get() + 1;

                if index_part[0] == '+' {
                    start_index += 1;
                }
                data_keeper.list = Self::parse_as_usize_sequence(
                    pattern,
                    pattern.get_subchars(start_index, index.len()),
                )?;
                index.set(index.len());
            }
            Self::Except => {
                data_keeper.except = Self::parse_as_usize_sequence(
                    pattern,
                    pattern.get_subchars(index.get() + 2, index.len()),
                )?;
                index.set(index.len());
            }
            Self::AnyWhere => {
                data_keeper.anywhere = Some(true);
                index.set(index.len());
            }
            Self::Greater => {
                let index_part = pattern.get_chars(index.get() + 1);
                let ret = Self::parse_as_usize(pattern, index_part)?;

                data_keeper.greater = Some(ret);
                index.set(index.len());
            }
            Self::Less => {
                let index_part = pattern.get_chars(index.get() + 1);
                let ret = Self::parse_as_usize(pattern, index_part)?;

                data_keeper.less = Some(ret);
                index.set(index.len());
            }
            Self::End => {
                debug!(?index, "State is End, index info");
                if !index.is_end() {
                    return Err(Error::opt_parsing_constructor_failed(pattern.get_pattern()));
                } else {
                    return Ok(true);
                }
            }
        }

        self.self_transition(index, pattern);

        self.parse(index, pattern, data_keeper)
    }

    // the index number is small in generally
    fn parse_as_usize<'pre>(pattern: &ParserPattern<'pre>, data: &[char]) -> Result<usize> {
        let mut count = 0;
        let mut ret = 0usize;

        for ch in data {
            // skip '+'
            if *ch == '+' || ch.is_ascii_whitespace() {
                continue;
            }
            count += 1;
            ret = ret * 10
                + ch.to_digit(10).ok_or_else(|| {
                    Error::opt_parsing_index_failed(
                        pattern.get_pattern().to_string(),
                        format!("{:?} is not a valid number", data),
                    )
                })? as usize;
        }
        if count == 0 {
            return Err(Error::opt_parsing_index_failed(
                pattern.get_pattern().to_string(),
                format!("{:?} is not a valid number", data),
            ));
        }
        Ok(ret)
    }

    fn parse_as_usize_sequence<'pre>(
        pattern: &ParserPattern<'pre>,
        data: &[char],
    ) -> Result<Vec<usize>> {
        let mut ret = vec![];
        let mut last = 0usize;

        for (index, ch) in data.iter().enumerate() {
            // skip '+'
            if *ch == '+' || ch.is_ascii_whitespace() {
                continue;
            }
            if *ch == ',' || *ch == ']' {
                if last == index {
                    return Err(Error::opt_parsing_index_failed(
                        pattern.get_pattern().to_string(),
                        format!("{:?} is not a valid number sequence", data),
                    ));
                }
                ret.push(Self::parse_as_usize(pattern, &data[last..index])?);
                last = index + 1;
            }
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod test {
    use super::parse_option_str;
    use crate::gstr;
    use crate::opt::index::Index;
    use ustr::Ustr;

    #[test]
    fn test_option_str_parser() {
        {
            // test 1
            let test_cases = vec![
                ("", None),
                (
                    "o=b",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b!",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b/",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b!/",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b!",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b/",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b!/",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b!",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b/",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b!/",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=b",
                    Some((None, None, Some(gstr("b")), Index::default(), None, None)),
                ),
                (
                    "=b!",
                    Some((
                        None,
                        None,
                        Some(gstr("b")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "=b/",
                    Some((
                        None,
                        None,
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "=b!/",
                    Some((
                        None,
                        None,
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=b/!",
                    Some((
                        None,
                        None,
                        Some(gstr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b@*",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@-1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@+42",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@>1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@<8",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(8),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@*",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@>1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@<8",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(8),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@*",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@>1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@<42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(42),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b!@*",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+42",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@>12",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(12),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@<42",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@*",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@>11",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(11),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@<4",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(4),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@*",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@<1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@>42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b/@1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@-1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@+42",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@*",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@>1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@<2",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(2),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@*",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@>1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@<42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@*",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@<11",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(11),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@>42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b!/@1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@-1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@+42",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@*",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@>1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@<42",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@*",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@>1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@<42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@-1",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@+42",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@*",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@>11",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(11),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@<4",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(4),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@*",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@>1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@<42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@*",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@>1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@<42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::less(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("o")),
                        Some(gstr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar!",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar/",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar!/",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar!",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar!/",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar!",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar!/",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=bar",
                    Some((None, None, Some(gstr("bar")), Index::default(), None, None)),
                ),
                (
                    "=bar!",
                    Some((
                        None,
                        None,
                        Some(gstr("bar")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "=bar/",
                    Some((
                        None,
                        None,
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "=bar!/",
                    Some((
                        None,
                        None,
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=bar/!",
                    Some((
                        None,
                        None,
                        Some(gstr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar@1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@-1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@+42",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@*",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@>1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@<42",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::less(42),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@*",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@>1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@<42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::less(42),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@*",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@>11",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::greater(11),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@<42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::less(42),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar!@1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+42",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar/@1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@-1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@+42",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar!/@1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@-1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@+42",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@-1",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@+42",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@[1, 2, 3]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@-1",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@+42",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@[1, 2, 3]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@+[4, 5, 12]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@-[1, 2, 4]",
                    Some((
                        Some(gstr("-")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@-1",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@+42",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@[1, 2, 3]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@+[4, 5, 12]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@-[1, 2, 4]",
                    Some((
                        Some(gstr("--")),
                        Some(gstr("option")),
                        Some(gstr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
            ];

            let prefixs = vec![gstr("--"), gstr("-")];

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &prefixs, &case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Ustr,
        prefix: &Vec<Ustr>,
        except: &Option<(
            Option<Ustr>,
            Option<Ustr>,
            Option<Ustr>,
            Index,
            Option<bool>,
            Option<bool>,
        )>,
    ) {
        let ret = parse_option_str(pattern, prefix);

        if let Ok(mut dk) = ret {
            assert!(except.is_some());

            if let Some(except) = except {
                let index = dk.gen_index();

                assert_eq!(except.0, dk.prefix);
                assert_eq!(except.1, dk.name);
                assert_eq!(except.2, dk.type_name);
                assert_eq!(except.3, index);
                assert_eq!(except.4, dk.deactivate);
                assert_eq!(except.5, dk.optional);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
