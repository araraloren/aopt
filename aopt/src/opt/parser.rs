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
/// [--option][=][type][!][@index]
///      |     |    |   |   |
///      |     |    |   |   |
///      |     |    |   |   |
///      |     |    |   |   The index part of option. Here are all the possible string:
///      |     |    |   |   @1 means first position
///      |     |    |   |   @-1 means last position
///      |     |    |   |   @[1, 2, 3] means the position 1, 2 and 3
///      |     |    |   |   @-[1, 2] means except the position 1, 2
///      |     |    |   |   @>2 means position that bigger than 2
///      |     |    |   |   @<3 means position less than 3
///      |     |    |   |   @* means all the position
///      |     |    |   |
///      |     |    |   Indicate the option is force required.
///      |     |    |
///      |     |    |
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
///     let ret = parser.parse("--aopt=t!".into())?;
///
///     assert_eq!(ret.name , Some(astr("--aopt")));
///     assert_eq!(ret.type_name, Some(astr("t")));
///     assert_eq!(ret.force, Some(true));
///     assert_eq!(ret.index, None);
///
///     let ret = parser.parse("bopt=t@[1,2,3]".into())?;
///
///     assert_eq!(ret.name , Some(astr("bopt")));
///     assert_eq!(ret.type_name, Some(astr("t")));
///     assert_eq!(ret.force, None);
///     assert_eq!(ret.idx(), Some(&Index::list(vec![1, 2, 3])));
///
/// #   Ok(())
/// # }
/// ```
///
/// For more examples, please reference test case [`test_option_str_parser`](../../src/aopt/set/parser.rs.html#542).
///
#[derive(Debug, Default)]
pub struct StrParser;

thread_local! {
    static STR_PARSER: Regex = Regex::new(r"^([^=]+)?(=([^=!@]+))?(!)?(@(.+))?$").unwrap();
}

impl StrParser {
    pub fn new() -> Self {
        Self {}
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

    pub fn parse_creator_string(&self, pattern: Str) -> Result<ConstrctInfo, Error> {
        let pattern_clone = pattern.clone();
        let pattern = pattern.as_str();

        STR_PARSER
            .try_with(|regex| {
                if let Some(cap) = regex.captures(pattern) {
                    let mut force = None;
                    let mut idx = None;

                    if let Some(mat) = cap.get(IDX_FORCE) {
                        match mat.as_str() {
                            "!" => {
                                force = Some(true);
                            }
                            _ => {
                                unreachable!("Oops ?!! Regex make sure option string correctly")
                            }
                        }
                    }
                    if let Some(mat) = cap.get(IDX_INDEX) {
                        idx = Some(Index::parse(mat.as_str())?);
                    }
                    Ok(ConstrctInfo::default()
                        .with_force(force)
                        .with_index(idx)
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
const IDX_FORCE: usize = 4;
const IDX_INDEX: usize = 6;

impl OptParser for StrParser {
    type Output = ConstrctInfo;

    type Error = Error;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error> {
        if pattern.trim().is_empty() {
            Ok(Self::Output::default())
        } else {
            self.parse_creator_string(pattern)
        }
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
                ("", Some((None, None, None, None))),
                ("o=b", Some((Some(astr("o")), Some(astr("b")), None, None))),
                (
                    "o=b!",
                    Some((Some(astr("o")), Some(astr("b")), None, Some(true))),
                ),
                (
                    "o=b!",
                    Some((Some(astr("o")), Some(astr("b")), None, Some(true))),
                ),
                (
                    "-o=b",
                    Some((Some(astr("-o")), Some(astr("b")), None, None)),
                ),
                (
                    "-o=b!",
                    Some((Some(astr("-o")), Some(astr("b")), None, Some(true))),
                ),
                (
                    "-/o=b",
                    Some((Some(astr("-/o")), Some(astr("b")), None, None)),
                ),
                (
                    "-/o=b!",
                    Some((Some(astr("-/o")), Some(astr("b")), None, Some(true))),
                ),
                (
                    "--o=b",
                    Some((Some(astr("--o")), Some(astr("b")), None, None)),
                ),
                (
                    "--o=b!",
                    Some((Some(astr("--o")), Some(astr("b")), None, Some(true))),
                ),
                (
                    "--/o=b",
                    Some((Some(astr("--/o")), Some(astr("b")), None, None)),
                ),
                (
                    "--/o=b!",
                    Some((Some(astr("--/o")), Some(astr("b")), None, Some(true))),
                ),
                ("=b", Some((None, Some(astr("b")), None, None))),
                ("=b!", Some((None, Some(astr("b")), None, Some(true)))),
                (
                    "o=b@*",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "o=b@1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "o=b@-1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "o=b@+42",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "o=b@1..",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        None,
                    )),
                ),
                (
                    "o=b@..8",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(8))),
                        None,
                    )),
                ),
                (
                    "o=b@[1, 2, 3]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "-o=b@*",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "-o=b@1",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "-o=b@-1",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "-o=b@+42",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "-o=b@1..",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        None,
                    )),
                ),
                (
                    "-o=b@..8",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(8))),
                        None,
                    )),
                ),
                (
                    "-o=b@[1, 2, 3]",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "-o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "-o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "--o=b@*",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "--o=b@1",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "--o=b@-1",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "--o=b@+42",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "--o=b@1..",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        None,
                    )),
                ),
                (
                    "--o=b@..42",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(42))),
                        None,
                    )),
                ),
                (
                    "--o=b@[1, 2, 3]",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "--o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "--o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "o=b!@*",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+42",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@12..",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(Some(12), None)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@..42",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(42))),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@1",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@-1",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@+42",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@*",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@11..",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::range(Some(11), None)),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@..4",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(4))),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("-o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@1",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@-1",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@+42",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@*",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@1..",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@..42",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(42))),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("--o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "o=b@1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "o=b@-1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "o=b@+42",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "o=b@*",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "o=b@1..",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        None,
                    )),
                ),
                (
                    "o=b@..2",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(2))),
                        None,
                    )),
                ),
                (
                    "o=b@[1, 2, 3]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "-/o=b@1",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "-/o=b@-1",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "-/o=b@+42",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "-/o=b@*",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "-/o=b@1..",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        None,
                    )),
                ),
                (
                    "-/o=b@..42",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(42))),
                        None,
                    )),
                ),
                (
                    "-/o=b@[1, 2, 3]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "-/o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "-/o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "--/o=b@1",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "--/o=b@-1",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "--/o=b@+42",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "--/o=b@*",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "--/o=b@11..",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::range(Some(11), None)),
                        None,
                    )),
                ),
                (
                    "--/o=b@..42",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(42))),
                        None,
                    )),
                ),
                (
                    "--/o=b@[1, 2, 3]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "--/o=b@+[4, 5, 12]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "--/o=b@-[1, 2, 4]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "o=b!@1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+42",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@*",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@1..",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@..42",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(42))),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@1",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@-1",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@+42",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@1",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@-1",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@+42",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@*",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@1..",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@..42",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(42))),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-1",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+42",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@*",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@11..",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(Some(11), None)),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@..4",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::range(None, Some(4))),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@6",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::forward(6)),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@-8",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::backward(8)),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@+22",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::forward(22)),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@*",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@1..9",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), Some(9))),
                        Some(true),
                    )),
                ),
                ("-/o=b!@6..2", None),
                (
                    "-/o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "-/o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("-/o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@1",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@-1",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@+42",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@*",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::anywhere()),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@1..",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::range(Some(1), None)),
                        Some(true),
                    )),
                ),
                ("--/o=b!@88..42", None),
                (
                    "--/o=b!@[1, 2, 3]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@+[4, 5, 12]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "--/o=b!@-[1, 2, 4]",
                    Some((
                        Some(astr("--/o")),
                        Some(astr("b")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "option=bar",
                    Some((Some(astr("option")), Some(astr("bar")), None, None)),
                ),
                (
                    "option=bar!",
                    Some((Some(astr("option")), Some(astr("bar")), None, Some(true))),
                ),
                (
                    "-option=bar",
                    Some((Some(astr("-option")), Some(astr("bar")), None, None)),
                ),
                (
                    "-option=bar!",
                    Some((Some(astr("-option")), Some(astr("bar")), None, Some(true))),
                ),
                (
                    "-/option=bar!",
                    Some((Some(astr("-/option")), Some(astr("bar")), None, Some(true))),
                ),
                (
                    "--option=bar",
                    Some((Some(astr("--option")), Some(astr("bar")), None, None)),
                ),
                (
                    "--option=bar!",
                    Some((Some(astr("--option")), Some(astr("bar")), None, Some(true))),
                ),
                (
                    "--/option=bar!",
                    Some((Some(astr("--/option")), Some(astr("bar")), None, Some(true))),
                ),
                ("=bar", Some((None, Some(astr("bar")), None, None))),
                ("=bar!", Some((None, Some(astr("bar")), None, Some(true)))),
                (
                    "option=bar@1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "option=bar@-1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "option=bar@+42",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "option=bar@*",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "option=bar@1..",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(1), None)),
                        None,
                    )),
                ),
                (
                    "option=bar@..42",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::range(None, Some(42))),
                        None,
                    )),
                ),
                (
                    "option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "-option=bar@1",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "-option=bar@-1",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "-option=bar@+42",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "-option=bar@*",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "-option=bar@1..8",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(1), Some(8))),
                        None,
                    )),
                ),
                (
                    "-option=bar@9..42",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(9), Some(42))),
                        None,
                    )),
                ),
                (
                    "-option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "-option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "-option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "--option=bar@1",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "--option=bar@-1",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "--option=bar@+42",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "--option=bar@*",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::anywhere()),
                        None,
                    )),
                ),
                (
                    "--option=bar@11..",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(11), None)),
                        None,
                    )),
                ),
                (
                    "--option=bar@42",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "--option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "--option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "--option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "option=bar!@1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+42",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@1",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@-1",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@+42",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "-option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("-option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@1",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@-1",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@+42",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "--option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("--option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "option=bar@1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "option=bar@-1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "option=bar@+42",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "-/option=bar@1",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "-/option=bar@-1",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "-/option=bar@+42",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "-/option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "-/option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "-/option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "--/option=bar@1",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        None,
                    )),
                ),
                (
                    "--/option=bar@-1",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        None,
                    )),
                ),
                (
                    "--/option=bar@+42",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        None,
                    )),
                ),
                (
                    "--/option=bar@[1, 2, 3]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        None,
                    )),
                ),
                (
                    "--/option=bar@+[4, 5, 12]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        None,
                    )),
                ),
                (
                    "--/option=bar@-[1, 2, 4]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        None,
                    )),
                ),
                (
                    "/option=bar!@1",
                    Some((
                        Some(astr("/option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "/option=bar!@-1",
                    Some((
                        Some(astr("/option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "/option=bar!@+42",
                    Some((
                        Some(astr("/option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "/option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "/option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "/option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("/option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@1",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@-1",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@+42",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@1",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@-1",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@+42",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-1",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+42",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@1..3",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(1), Some(3))),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@1",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@-1",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@+42",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@2..4",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(2), Some(4))),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@2..2",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(2), Some(2))),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@..5",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::range(None, Some(5))),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@6..",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(6), None)),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "-/option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("-/option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@1",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::forward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@-1",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::backward(1)),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@+42",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::forward(42)),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@3..",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(3), None)),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@1..2",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::range(Some(1), Some(2))),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@..7",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::range(None, Some(7))),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@[1, 2, 3]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![1, 2, 3])),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@+[4, 5, 12]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::list(vec![4, 5, 12])),
                        Some(true),
                    )),
                ),
                (
                    "--/option=bar!@-[1, 2, 4]",
                    Some((
                        Some(astr("--/option")),
                        Some(astr("bar")),
                        Some(Index::except(vec![1, 2, 4])),
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
        except: &Option<(Option<Str>, Option<Str>, Option<Index>, Option<bool>)>,
    ) {
        let ret = parser.parse(pattern);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            if let Some(except) = except {
                let index = dk.idx();

                assert_eq!(except.0, dk.name);
                assert_eq!(except.1, dk.type_name);
                assert_eq!(except.2.as_ref(), index);
                assert_eq!(except.3, dk.force);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
