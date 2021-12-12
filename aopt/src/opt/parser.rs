use ustr::Ustr;

use super::index::Index;

use crate::err::ConstructError;
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
/// use aopt::Ustr;
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
/// For more examples, please reference test case [`test_option_str_parser`](../../../src/aopt/opt/parser.rs.html#538).
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
    Err(ConstructError::ParsingFailed(pattern.get_pattern().to_owned()).into())
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

    pub forward_index: Option<u64>,

    pub backward_index: Option<u64>,

    pub anywhere: Option<bool>,

    pub list: Vec<u64>,

    pub except: Vec<u64>,

    pub greater: Option<u64>,

    pub less: Option<u64>,
}

impl DataKeeper {
    pub fn gen_index(&mut self) -> Index {
        if self.forward_index.is_some() {
            Index::forward(self.forward_index.unwrap())
        } else if self.backward_index.is_some() {
            Index::backward(self.backward_index.unwrap())
        } else if self.anywhere.unwrap_or(false) {
            Index::anywhere()
        } else if self.list.len() > 0 {
            Index::list(std::mem::take(&mut self.list))
        } else if self.except.len() > 0 {
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
            || self.list.len() > 0
            || self.except.len() > 0
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

    pub fn self_transition<'vec, 'pre>(
        &mut self,
        index: &ParseIndex,
        pattern: &ParserPattern<'pre>,
    ) {
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
                if let Some(ch) = pattern.chars(index.get()).nth(0) {
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

                if index_part.starts_with("+[") || index_part.starts_with("[") {
                    Self::List
                } else if index_part.starts_with("-[") {
                    Self::Except
                } else if index_part.starts_with("-") {
                    Self::BackwardIndex
                } else if index_part == Self::anywhere_symbol() {
                    Self::AnyWhere
                } else if index_part.starts_with(">") {
                    Self::Greater
                } else if index_part.starts_with("<") {
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
                for prefix in pattern.get_prefixs() {
                    if pattern.get_pattern().starts_with(prefix.as_ref()) {
                        data_keeper.prefix = Some(prefix.clone());
                        index.inc(prefix.len());
                        break;
                    }
                }
            }
            Self::Name => {
                let start = index.get();

                for (cur, ch) in pattern.chars(start).enumerate() {
                    let mut name_end = 0;

                    if ch == '=' || ch == '!' || ch == '/' || ch == '@' {
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
                        let name = pattern.get_pattern().get(start..name_end);

                        if let Some(name) = name {
                            data_keeper.name = Some(name.into());
                            index.set(name_end);
                        } else {
                            error!(
                                ?pattern,
                                "accessing string [{}, {}) failed", start, name_end
                            );
                            return Err(ConstructError::PatternAccessFailed(
                                pattern.get_pattern().to_owned(),
                                start,
                                name_end,
                            )
                            .into());
                        }
                        break;
                    }
                }
            }
            Self::Equal => {
                index.inc(1);
            }
            Self::Type => {
                let start = index.get();

                for (cur, ch) in pattern.chars(start).enumerate() {
                    let mut type_end = 0;

                    if ch == '!' || ch == '/' || ch == '@' {
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
                        let type_ = pattern.get_pattern().get(start..type_end);

                        if let Some(type_) = type_ {
                            data_keeper.type_name = Some(type_.into());
                            index.set(type_end);
                        } else {
                            error!(
                                ?pattern,
                                "accessing string [{}, {}) failed", start, type_end
                            );
                            return Err(ConstructError::PatternAccessFailed(
                                pattern.get_pattern().to_owned(),
                                start,
                                type_end,
                            )
                            .into());
                        }

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
                let (_, index_part) = pattern.get_pattern().split_at(index.get());

                let ret = index_part.parse::<u64>().map_err(|e| {
                    ConstructError::IndexParsingFailed(
                        pattern.get_pattern().to_owned(),
                        format!("{:?}", e),
                    )
                })?;
                if ret > 0 {
                    data_keeper.forward_index = Some(ret);
                } else {
                    data_keeper.anywhere = Some(true);
                }
                index.set(index.len());
            }
            Self::BackwardIndex => {
                let (_, index_part) = pattern.get_pattern().split_at(index.get() + 1);

                let ret = index_part.parse::<u64>().map_err(|e| {
                    ConstructError::IndexParsingFailed(
                        pattern.get_pattern().to_owned(),
                        format!("{:?}", e),
                    )
                })?;
                if ret > 0 {
                    data_keeper.backward_index = Some(ret);
                } else {
                    data_keeper.anywhere = Some(true);
                }
                index.set(index.len());
            }
            Self::List => {
                let (_, index_part) = pattern.get_pattern().split_at(index.get());

                if index_part.starts_with("+[") {
                    let index_part = pattern
                        .get_pattern()
                        .get(index.get() + 2..index.len() - 1)
                        .unwrap();

                    data_keeper.list = index_part
                        .split(',')
                        .map(|v| {
                            v.trim().parse::<u64>().map_err(|e| {
                                ConstructError::IndexParsingFailed(
                                    pattern.get_pattern().to_owned(),
                                    format!("{:?}", e),
                                )
                                .into()
                            })
                        })
                        .collect::<Result<Vec<u64>>>()?;
                } else {
                    let index_part = pattern
                        .get_pattern()
                        .get(index.get() + 1..index.len() - 1)
                        .unwrap();

                    data_keeper.list = index_part
                        .split(',')
                        .map(|v| {
                            v.trim().parse::<u64>().map_err(|e| {
                                ConstructError::IndexParsingFailed(
                                    pattern.get_pattern().to_owned(),
                                    format!("{:?}", e),
                                )
                                .into()
                            })
                        })
                        .collect::<Result<Vec<u64>>>()?;
                }
                index.set(index.len());
            }
            Self::Except => {
                let index_part = pattern
                    .get_pattern()
                    .get(index.get() + 2..index.len() - 1)
                    .unwrap();

                data_keeper.except = index_part
                    .split(',')
                    .map(|v| {
                        v.trim().parse::<u64>().map_err(|e| {
                            ConstructError::IndexParsingFailed(
                                pattern.get_pattern().to_owned(),
                                format!("{:?}", e),
                            )
                            .into()
                        })
                    })
                    .collect::<Result<Vec<u64>>>()?;
                index.set(index.len());
            }
            Self::AnyWhere => {
                data_keeper.anywhere = Some(true);
                index.set(index.len());
            }
            Self::Greater => {
                let (_, index_part) = pattern.get_pattern().split_at(index.get() + 1);

                let ret = index_part.parse::<u64>().map_err(|e| {
                    ConstructError::IndexParsingFailed(
                        pattern.get_pattern().to_owned(),
                        format!("{:?}", e),
                    )
                })?;
                data_keeper.greater = Some(ret);
                index.set(index.len());
            }
            Self::Less => {
                let (_, index_part) = pattern.get_pattern().split_at(index.get() + 1);

                let ret = index_part.parse::<u64>().map_err(|e| {
                    ConstructError::IndexParsingFailed(
                        pattern.get_pattern().to_owned(),
                        format!("{:?}", e),
                    )
                })?;
                data_keeper.less = Some(ret);
                index.set(index.len());
            }
            Self::End => {
                debug!(?index, "!!!!!!!!!!!!!!");
                if !index.is_end() {
                    return Err(
                        ConstructError::ParsingFailed(pattern.get_pattern().to_owned()).into(),
                    );
                } else {
                    return Ok(true);
                }
            }
        }

        self.self_transition(index, pattern);

        self.parse(index, pattern, data_keeper)
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
                    Some((None, Some("o"), Some("b"), Index::default(), false, true)),
                ),
                (
                    "o=b!",
                    Some((None, Some("o"), Some("b"), Index::default(), false, false)),
                ),
                (
                    "o=b/",
                    Some((None, Some("o"), Some("b"), Index::default(), true, true)),
                ),
                (
                    "o=b!/",
                    Some((None, Some("o"), Some("b"), Index::default(), true, false)),
                ),
                (
                    "o=b/!",
                    Some((None, Some("o"), Some("b"), Index::default(), true, false)),
                ),
                (
                    "-o=b",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b!",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b/",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b!/",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b!",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b/",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b!/",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "=b",
                    Some((None, None, Some("b"), Index::default(), false, true)),
                ),
                (
                    "=b!",
                    Some((None, None, Some("b"), Index::default(), false, false)),
                ),
                (
                    "=b/",
                    Some((None, None, Some("b"), Index::default(), true, true)),
                ),
                (
                    "=b!/",
                    Some((None, None, Some("b"), Index::default(), true, false)),
                ),
                (
                    "=b/!",
                    Some((None, None, Some("b"), Index::default(), true, false)),
                ),
                (
                    "o=b@1",
                    Some((None, Some("o"), Some("b"), Index::forward(1), false, true)),
                ),
                (
                    "o=b@-1",
                    Some((None, Some("o"), Some("b"), Index::backward(1), false, true)),
                ),
                (
                    "o=b@+42",
                    Some((None, Some("o"), Some("b"), Index::forward(42), false, true)),
                ),
                (
                    "o=b@[1, 2, 3]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b@+[4, 5, 12]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b@-[1, 2, 4]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b!@1",
                    Some((None, Some("o"), Some("b"), Index::forward(1), false, false)),
                ),
                (
                    "o=b!@-1",
                    Some((None, Some("o"), Some("b"), Index::backward(1), false, false)),
                ),
                (
                    "o=b!@+42",
                    Some((None, Some("o"), Some("b"), Index::forward(42), false, false)),
                ),
                (
                    "o=b!@[1, 2, 3]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b!@+[4, 5, 12]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b!@-[1, 2, 4]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b/@1",
                    Some((None, Some("o"), Some("b"), Index::forward(1), true, true)),
                ),
                (
                    "o=b/@-1",
                    Some((None, Some("o"), Some("b"), Index::backward(1), true, true)),
                ),
                (
                    "o=b/@+42",
                    Some((None, Some("o"), Some("b"), Index::forward(42), true, true)),
                ),
                (
                    "o=b/@[1, 2, 3]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b/@+[4, 5, 12]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b/@-[1, 2, 4]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b!/@1",
                    Some((None, Some("o"), Some("b"), Index::forward(1), true, false)),
                ),
                (
                    "o=b!/@-1",
                    Some((None, Some("o"), Some("b"), Index::backward(1), true, false)),
                ),
                (
                    "o=b!/@+42",
                    Some((None, Some("o"), Some("b"), Index::forward(42), true, false)),
                ),
                (
                    "o=b!/@[1, 2, 3]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b!/@+[4, 5, 12]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b!/@-[1, 2, 4]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@1",
                    Some((None, Some("o"), Some("b"), Index::forward(1), true, false)),
                ),
                (
                    "o=b/!@-1",
                    Some((None, Some("o"), Some("b"), Index::backward(1), true, false)),
                ),
                (
                    "o=b/!@+42",
                    Some((None, Some("o"), Some("b"), Index::forward(42), true, false)),
                ),
                (
                    "o=b/!@[1, 2, 3]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@+[4, 5, 12]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@-[1, 2, 4]",
                    Some((
                        None,
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar!",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar/",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar!/",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar!",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar/",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar!/",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar!",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar/",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar!/",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "=bar",
                    Some((None, None, Some("bar"), Index::default(), false, true)),
                ),
                (
                    "=bar!",
                    Some((None, None, Some("bar"), Index::default(), false, false)),
                ),
                (
                    "=bar/",
                    Some((None, None, Some("bar"), Index::default(), true, true)),
                ),
                (
                    "=bar!/",
                    Some((None, None, Some("bar"), Index::default(), true, false)),
                ),
                (
                    "=bar/!",
                    Some((None, None, Some("bar"), Index::default(), true, false)),
                ),
                (
                    "option=bar@1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@-1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@+42",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@[1, 2, 3]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@+[4, 5, 12]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@-[1, 2, 4]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar!@1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@-1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@+42",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@[1, 2, 3]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@+[4, 5, 12]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@-[1, 2, 4]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar/@1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@-1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@+42",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@[1, 2, 3]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@+[4, 5, 12]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@-[1, 2, 4]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar!/@1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@-1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@+42",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@[1, 2, 3]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@+[4, 5, 12]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@-[1, 2, 4]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@-1",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@+42",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@[1, 2, 3]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@+[4, 5, 12]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@-[1, 2, 4]",
                    Some((
                        None,
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
            ];

            let prefixs = vec![gstr("--"), gstr("-")];

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &prefixs, &case.1);
            }
        }
        {
            // test 2
            let test_cases = vec![
                ("", None),
                (
                    "o=b",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b!",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b/",
                    Some((Some(""), Some("o"), Some("b"), Index::default(), true, true)),
                ),
                (
                    "o=b!/",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b!",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b/",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b!/",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b!",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b/",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b!/",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "=b",
                    Some((Some(""), None, Some("b"), Index::default(), false, true)),
                ),
                (
                    "=b!",
                    Some((Some(""), None, Some("b"), Index::default(), false, false)),
                ),
                (
                    "=b/",
                    Some((Some(""), None, Some("b"), Index::default(), true, true)),
                ),
                (
                    "=b!/",
                    Some((Some(""), None, Some("b"), Index::default(), true, false)),
                ),
                (
                    "=b/!",
                    Some((Some(""), None, Some("b"), Index::default(), true, false)),
                ),
                (
                    "o=b@1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b@-1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b@+42",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "-o=b@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "--o=b@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "o=b!@1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b!@-1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b!@+42",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b!@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b!@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b!@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "-o=b!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "--o=b!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "o=b/@1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b/@-1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b/@+42",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b/@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b/@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b/@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "-o=b/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "--o=b/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "o=b!/@1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b!/@-1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b!/@+42",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b!/@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b!/@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b!/@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b!/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b!/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@-1",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@+42",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "o=b/!@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@-1",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@+42",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-o=b/!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@-1",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@+42",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--o=b/!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("o"),
                        Some("b"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar!",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar/",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar!/",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar!",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar/",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar!/",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar!",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar/",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar!/",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::default(),
                        true,
                        false,
                    )),
                ),
                (
                    "=bar",
                    Some((Some(""), None, Some("bar"), Index::default(), false, true)),
                ),
                (
                    "=bar!",
                    Some((Some(""), None, Some("bar"), Index::default(), false, false)),
                ),
                (
                    "=bar/",
                    Some((Some(""), None, Some("bar"), Index::default(), true, true)),
                ),
                (
                    "=bar!/",
                    Some((Some(""), None, Some("bar"), Index::default(), true, false)),
                ),
                (
                    "=bar/!",
                    Some((Some(""), None, Some("bar"), Index::default(), true, false)),
                ),
                (
                    "option=bar@1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@-1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@+42",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "-option=bar@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        true,
                    )),
                ),
                (
                    "--option=bar@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        true,
                    )),
                ),
                (
                    "option=bar!@1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@-1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@+42",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar!@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "-option=bar!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        false,
                        false,
                    )),
                ),
                (
                    "--option=bar!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        false,
                        false,
                    )),
                ),
                (
                    "option=bar/@1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@-1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@+42",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar/@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "-option=bar/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        true,
                    )),
                ),
                (
                    "--option=bar/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        true,
                    )),
                ),
                (
                    "option=bar!/@1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@-1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@+42",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar!/@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar!/@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar!/@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@-1",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@+42",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@[1, 2, 3]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@+[4, 5, 12]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "option=bar/!@-[1, 2, 4]",
                    Some((
                        Some(""),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@-1",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@+42",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@[1, 2, 3]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@+[4, 5, 12]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "-option=bar/!@-[1, 2, 4]",
                    Some((
                        Some("-"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@-1",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::backward(1),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@+42",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::forward(42),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@[1, 2, 3]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![1, 2, 3]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@+[4, 5, 12]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::list(vec![4, 5, 12]),
                        true,
                        false,
                    )),
                ),
                (
                    "--option=bar/!@-[1, 2, 4]",
                    Some((
                        Some("--"),
                        Some("option"),
                        Some("bar"),
                        Index::except(vec![1, 2, 4]),
                        true,
                        false,
                    )),
                ),
            ];

            let prefixs = vec![gstr("--"), gstr("-"), gstr("")];

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &prefixs, &case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Ustr,
        prefix: &Vec<Ustr>,
        except: &Option<(Option<&str>, Option<&str>, Option<&str>, Index, bool, bool)>,
    ) {
        let ret = parse_option_str(pattern, prefix);

        if let Ok(mut dk) = ret {
            assert!(except.is_some());

            let default = gstr("");

            if let Some(except) = except {
                let index = dk.gen_index();

                assert_eq!(
                    except.0.unwrap_or(""),
                    dk.prefix.unwrap_or(default).as_ref()
                );
                assert_eq!(
                    except.1.unwrap_or(""),
                    dk.name.unwrap_or(default.clone()).as_ref()
                );
                assert_eq!(
                    except.2.unwrap_or(""),
                    dk.type_name.unwrap_or(default.clone()).as_ref()
                );
                assert_eq!(except.3, index);
                assert_eq!(except.4, dk.deactivate.unwrap_or(false));
                assert_eq!(!except.5, dk.optional.unwrap_or(false));
            }
        } else {
            assert!(except.is_none());
        }
    }
}
