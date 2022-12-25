use regex::Regex;

use super::{ConstrctInfo, OptParser};
use crate::opt::Index;
use crate::Error;
use crate::Str;

/// Parse the option string with given prefixs, return an [`ConstrctInfo`].
///
/// The struct of the option string are:
///
/// ```!
/// [--option][=][type][/][!][@index]
///      |     |    |   |  |   |
///      |     |    |   |  |   |
///      |     |    |   |  |   |
///      |     |    |   |  |   The index part of option. Here are all the possible string:
///      |     |    |   |  |   @1 means first position
///      |     |    |   |  |   @-1 means last position
///      |     |    |   |  |   @[1, 2, 3] means the position 1, 2 and 3
///      |     |    |   |  |   @-[1, 2] means except the position 1, 2
///      |     |    |   |  |   @>2 means position that bigger than 2
///      |     |    |   |  |   @<3 means position less than 3
///      |     |    |   |  |   @* means all the position
///      |     |    |   |  |
///      |     |    |   |  Indicate the option is force required.
///      |     |    |   |
///      |     |    |   The disable symbol, generally it is using for boolean option.
///      |     |    |
///      |     |    The type name of option.
///      |     |    
///      |     The delimiter of option name and type.
///      |
///      The option name part, it must be provide by user.
/// ```
///
/// # Example
///
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::astr;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
///     let parser = StrParser::default();
///     let ret = parser.parse("--aopt=t!/".into())?;
///
///     assert_eq!(ret.prefix, Some(astr("--")));
///     assert_eq!(ret.name , Some(astr("aopt")));
///     assert_eq!(ret.type_name, Some(astr("t")));
///     assert_eq!(ret.deactivate, Some(true));
///     assert_eq!(ret.optional, Some(true));
///     assert_eq!(ret.index, None);
///
///     let ret = parser.parse("bopt=t@[1,2,3]".into())?;
///
///     assert_eq!(ret.prefix, None);
///     assert_eq!(ret.name , Some(astr("bopt")));
///     assert_eq!(ret.type_name, Some(astr("t")));
///     assert_eq!(ret.deactivate, None);
///     assert_eq!(ret.optional, None);
///     assert_eq!(ret.idx(), Some(&Index::list(vec![1, 2, 3])));
///
/// #   Ok(())
/// # }
/// ```
///
/// For more examples, please reference test case [`test_option_str_parser`](../../src/aopt/set/parser.rs.html#542).
///
#[derive(Debug, Default)]
pub struct StrParser {
    prefix: Vec<Str>,
}

thread_local! {
    static STR_PARSER: Regex = Regex::new(r"^([^=]+)?(=([^=/!@]+))?([!/])?([!/])?(@(.+))?$").unwrap();
}

impl StrParser {
    pub fn new() -> Self {
        Self { prefix: vec![] }
    }

    // the index number is small in generally
    pub(crate) fn parse_as_usize(pattern: &str, data: &str) -> Result<usize, Error> {
        let mut count = 0;
        let mut ret = 0usize;

        for ch in data.chars() {
            // skip '+'
            if ch == '+' || ch.is_ascii_whitespace() {
                continue;
            }
            count += 1;
            ret = ret * 10
                + ch.to_digit(10).ok_or_else(|| {
                    Error::con_parsing_index_failed(
                        pattern.to_string(),
                        format!("{} is not a valid number", data),
                    )
                })? as usize;
        }
        if count == 0 {
            return Err(Error::con_parsing_index_failed(
                pattern.to_string(),
                format!("{} is not a valid number", data),
            ));
        }
        Ok(ret)
    }

    pub(crate) fn parse_as_usize_sequence(pattern: &str, data: &str) -> Result<Vec<usize>, Error> {
        let mut ret = vec![];
        let mut last = 0usize;

        for (index, ch) in data.chars().enumerate() {
            // skip '+'
            if ch == '+' || ch == '[' {
                last += 1;
                continue;
            }
            if ch.is_ascii_whitespace() {
                continue;
            }
            if ch == ',' || ch == ']' {
                if last == index {
                    return Err(Error::con_parsing_index_failed(
                        pattern.to_string(),
                        format!("{} is not a valid number", data),
                    ));
                }
                ret.push(Self::parse_as_usize(pattern, &data[last..index])?);
                last = index + 1;
            }
        }
        Ok(ret)
    }

    pub fn parse_creator_string(&self, pattern: Str, prefix: Str) -> Result<ConstrctInfo, Error> {
        let (_, left_part) = pattern.split_at(prefix.len());
        let pattern_clone = pattern.clone();

        STR_PARSER
            .try_with(|regex| {
                if let Some(cap) = regex.captures(left_part) {
                    let mut deactivate = None;
                    let mut optional = None;
                    let mut opt_index = None;

                    for index in [IDX_DEAC, IDX_OPTN] {
                        if let Some(mat) = cap.get(index) {
                            match mat.as_str() {
                                "!" => {
                                    optional = Some(true);
                                }
                                "/" => {
                                    deactivate = Some(true);
                                }
                                _ => {
                                    return Err(Error::raise_error(format!(
                                        "Index syntax error, except ! or /, found {}",
                                        mat.as_str()
                                    )))
                                }
                            }
                        }
                    }
                    if let Some(mat) = cap.get(IDX_INDEX) {
                        opt_index = Some(Index::parse(mat.as_str())?);
                    }
                    Ok(ConstrctInfo::default()
                        .with_pre(Some(prefix))
                        .with_deact(deactivate)
                        .with_opt(optional)
                        .with_index(opt_index)
                        .with_pat(pattern_clone)
                        .with_name(cap.get(IDX_NAME).map(|v| Str::from(v.as_str())))
                        .with_ty(cap.get(IDX_TYPE).map(|v| Str::from(v.as_str()))))
                } else {
                    Err(Error::con_parsing_failed(pattern_clone))
                }
            })
            .map_err(|e| Error::raise_error(format!("Can not access str parser regex: {:?}", e)))?
    }
}

const IDX_NAME: usize = 1;
const IDX_TYPE: usize = 3;
const IDX_DEAC: usize = 4;
const IDX_OPTN: usize = 5;
const IDX_INDEX: usize = 7;

impl OptParser for StrParser {
    type Output = ConstrctInfo;

    type Error = Error;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error> {
        if pattern.trim().is_empty() {
            return Ok(Self::Output::default());
        } else {
            for prefix in self.prefix.iter() {
                if pattern.starts_with(prefix.as_str()) {
                    if let Ok(data_keeper) =
                        self.parse_creator_string(pattern.clone(), prefix.clone())
                    {
                        return Ok(data_keeper);
                    }
                }
            }
            // pass en empty prefix to the parser
            if let Ok(mut data_keeper) = self.parse_creator_string(pattern.clone(), Str::from("")) {
                data_keeper.prefix = None;
                return Ok(data_keeper);
            }
        }
        Err(Error::con_parsing_failed(pattern))
    }
}

#[cfg(test)]
mod test {
    use super::StrParser;
    use crate::astr;
    use crate::opt::Index;
    use crate::opt::Information;
    use crate::opt::OptParser;
    use crate::Str;

    #[test]
    fn test_option_str_parser() {
        {
            // test 1
            let test_cases = vec![
                ("", Some((None, None, None, Index::default(), None, None))),
                (
                    "o=b",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b!",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b/",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b!/",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b!",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b/",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b!/",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b!",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b/",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b!/",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=b",
                    Some((None, None, Some(astr("b")), Index::default(), None, None)),
                ),
                (
                    "=b!",
                    Some((
                        None,
                        None,
                        Some(astr("b")),
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
                        Some(astr("b")),
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
                        Some(astr("b")),
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
                        Some(astr("b")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b@*",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@-1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@+42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@1..",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), None),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@..8",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(8)),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@*",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@1..3",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), Some(3)),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@2..8",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(2), Some(8)),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@*",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@1..8",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), Some(8)),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@..42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(42)),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b!@*",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@..12",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(12)),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@..42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(42)),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@*",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@11..",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(11), None),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@..4",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(4)),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@*",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@..1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(1)),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@42..",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(42), None),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b/@1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@-1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@+42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@*",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@1..",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), None),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@..2",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(2)),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@*",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@1..",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), None),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@..42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(42)),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@*",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@..11",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(11)),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@42..",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(42), None),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b!/@1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@-1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@+42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@*",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@1..12",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), Some(12)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@..42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(42)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!/@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@*",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@1..",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), None),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@..42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(42)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@-1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@+42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@*",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@11..",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(11), None),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@..4",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(4)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@*",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@1..42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), Some(42)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@6..42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(6), Some(42)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@*",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@1..12",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(Some(1), Some(12)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@..42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::range(None, Some(42)),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar!",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar/",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar!/",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar!",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar!/",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar!",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar!/",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=bar",
                    Some((None, None, Some(astr("bar")), Index::default(), None, None)),
                ),
                (
                    "=bar!",
                    Some((
                        None,
                        None,
                        Some(astr("bar")),
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
                        Some(astr("bar")),
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
                        Some(astr("bar")),
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
                        Some(astr("bar")),
                        Index::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar@1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@-1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@+42",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@*",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@1..166",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::range(Some(1), Some(166)),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@8..42",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::range(Some(8), Some(42)),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@*",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@1..",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::range(Some(1), None),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@..42",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::range(None, Some(42)),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@*",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@11..",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::range(Some(11), None),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@..82",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::range(None, Some(82)),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar!@1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+42",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "option=bar/@1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@-1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@+42",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-option=bar/@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--option=bar/@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "option=bar!/@1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@-1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@+42",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!/@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!/@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!/@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@-1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@+42",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@[1, 2, 3]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@+[4, 5, 12]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "option=bar/!@-[1, 2, 4]",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@-1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@+42",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@[1, 2, 3]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@+[4, 5, 12]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar/!@-[1, 2, 4]",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@-1",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::backward(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@+42",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::forward(42),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@[1, 2, 3]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![1, 2, 3]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@+[4, 5, 12]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::list(vec![4, 5, 12]),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar/!@-[1, 2, 4]",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        Index::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
            ];
            let parser = StrParser::default();

            for case in test_cases.iter() {
                try_to_verify_one_task(astr(case.0), &parser, &case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Str,
        parser: &StrParser,
        except: &Option<(
            Option<Str>,
            Option<Str>,
            Option<Str>,
            Index,
            Option<bool>,
            Option<bool>,
        )>,
    ) {
        let ret = parser.parse(pattern);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            if let Some(except) = except {
                let index = dk.idx();

                assert_eq!(except.0, dk.prefix);
                assert_eq!(except.1, dk.name);
                assert_eq!(except.2, dk.type_name);
                assert_eq!(Some(&except.3), index.or(Some(&Index::default())));
                assert_eq!(except.4, dk.deactivate);
                assert_eq!(except.5, dk.optional);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
