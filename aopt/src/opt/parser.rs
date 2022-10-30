use regex::Regex;

use super::{ConstrctInfo, OptParser};
use crate::set::Pre;
use crate::Error;
use crate::Str;

/// Parse the option string with given prefixs, return an [`OptConstrctInfo`].
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
/// extern crate aopt as test_crate;
///
/// use test_crate::astr;
/// use test_crate::err::Result;
/// use test_crate::opt::OptStringParser;
/// use test_crate::opt::OptParser;
/// use test_crate::opt::OptIndex;
/// use test_crate::opt::Information;
///
/// fn main() -> Result<()> {
///     let parser = OptStringParser::default().with_pre("--");
///     let ret = parser.parse("--aopt=t!/".into())?;
///
///     assert_eq!(ret.prefix, Some(astr("--")));
///     assert_eq!(ret.name , Some(astr("aopt")));
///     assert_eq!(ret.type_name, Some(astr("t")));
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
///     let ret = parser.parse("bopt=t@[1,2,3]".into())?;
///
///     assert_eq!(ret.prefix, None);
///     assert_eq!(ret.name , Some(astr("bopt")));
///     assert_eq!(ret.type_name, Some(astr("t")));
///     assert_eq!(ret.deactivate, None);
///     assert_eq!(ret.optional, None);
///     assert_eq!(ret.forward_index, None);
///     assert_eq!(ret.backward_index, None);
///     assert_eq!(ret.anywhere, None);
///     assert_eq!(ret.list, []);
///     assert_eq!(ret.except, []);
///     assert_eq!(ret.greater, None);
///     assert_eq!(ret.less, None);
///     assert_eq!(ret.idx(), Some(&OptIndex::list(vec![1, 2, 3])));
///
///     Ok(())
/// }
/// ```
///
/// For more examples, please reference test case [`test_option_str_parser`](../../src/aopt/set/parser.rs.html#542).
///
#[derive(Debug)]
pub struct StrParser {
    regex: Regex,
    prefix: Vec<Str>,
}

impl Default for StrParser {
    fn default() -> Self {
        let regex = Regex::new(r"^([^=]+)?(=([^=/!@]+))?([!/])?([!/])?(@(?:([+-><])?(\d+)|([+-])?(\[(?:\s*\d+,?\s*)+\])|(\*)))?$").unwrap();
        Self::new(regex)
    }
}

impl Pre for StrParser {
    fn prefix(&self) -> &[Str] {
        &self.prefix
    }

    fn add_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.prefix.push(prefix.into());
        self.prefix.sort_by_key(|b| std::cmp::Reverse(b.len()));
        self
    }
}

impl StrParser {
    pub fn new(regex: Regex) -> Self {
        Self {
            regex,
            prefix: vec![],
        }
    }

    pub fn with_pre(mut self, prefix: &str) -> Self {
        self.add_prefix(prefix);
        self
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    pub fn rem_pre(&mut self, prefix: &str) -> &mut Self {
        for (idx, value) in self.prefix.iter().enumerate() {
            if *value == prefix {
                self.prefix.remove(idx);
                break;
            }
        }
        self
    }

    // the index number is small in generally
    fn parse_as_usize(pattern: &Str, data: &str) -> Result<usize, Error> {
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

    fn parse_as_usize_sequence(pattern: &Str, data: &str) -> Result<Vec<usize>, Error> {
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

        if let Some(cap) = self.regex().captures(left_part) {
            let mut deactivate = None;
            let mut optional = None;
            let mut forward_index = None;
            let mut backward_index = None;
            let mut list = vec![];
            let mut except = vec![];
            let mut greater = None;
            let mut less = None;
            let anywhere = cap.get(IDX_ANY).map(|_| true);

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
                            panic!("Oops!? Where are you going!")
                        }
                    }
                }
            }
            if let Some(value_mat) = cap.get(IDX_IDX1) {
                forward_index = Some(Self::parse_as_usize(&pattern, value_mat.as_str())?);
            }
            if let Some(list_mat) = cap.get(IDX_IDX2) {
                list = Self::parse_as_usize_sequence(&pattern, list_mat.as_str())?;
            }
            if let Some(mat) = cap.get(IDX_SIGN1) {
                match mat.as_str() {
                    "+" | "" => {}
                    "-" => {
                        backward_index = forward_index;
                        forward_index = None;
                    }
                    ">" => {
                        greater = forward_index;
                        forward_index = None;
                    }
                    "<" => {
                        less = forward_index;
                        forward_index = None;
                    }
                    _ => {
                        panic!("Oops!? Where are you going!")
                    }
                }
            }
            if let Some(mat) = cap.get(IDX_SIGN2) {
                match mat.as_str() {
                    "+" | "" => {}
                    "-" => {
                        except = list;
                        list = vec![];
                    }
                    _ => {
                        panic!("Oops!? Where are you going!")
                    }
                }
            }
            Ok(ConstrctInfo::default()
                .with_pre(Some(prefix))
                .with_deact(deactivate)
                .with_opt(optional)
                .with_fwd(forward_index)
                .with_bwd(backward_index)
                .with_aw(anywhere)
                .with_ls(list)
                .with_exp(except)
                .with_gt(greater)
                .with_le(less)
                .with_pat(pattern.clone())
                .with_name(cap.get(IDX_NAME).map(|v| Str::from(v.as_str())))
                .with_ty(cap.get(IDX_TYPE).map(|v| Str::from(v.as_str()))))
        } else {
            Err(Error::con_parsing_failed(pattern))
        }
    }
}

const IDX_NAME: usize = 1;
const IDX_TYPE: usize = 3;
const IDX_DEAC: usize = 4;
const IDX_OPTN: usize = 5;
const IDX_SIGN1: usize = 7;
const IDX_IDX1: usize = 8;
const IDX_SIGN2: usize = 9;
const IDX_IDX2: usize = 10;
const IDX_ANY: usize = 11;

impl OptParser for StrParser {
    type Output = ConstrctInfo;

    type Error = Error;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error> {
        if pattern.trim().is_empty() {
            return Ok(Self::Output::default());
        } else {
            for prefix in self.prefix.iter() {
                if pattern.starts_with(prefix.as_str()) {
                    if let Ok(mut data_keeper) =
                        self.parse_creator_string(pattern.clone(), prefix.clone())
                    {
                        data_keeper.gen_idx();
                        return Ok(data_keeper);
                    }
                }
            }
            // pass en empty prefix to the parser
            if let Ok(mut data_keeper) = self.parse_creator_string(pattern.clone(), Str::from("")) {
                data_keeper.prefix = None;
                data_keeper.gen_idx();
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
    use crate::opt::Information;
    use crate::opt::OptIndex;
    use crate::opt::OptParser;
    use crate::Str;

    #[test]
    fn test_option_str_parser() {
        {
            // test 1
            let test_cases = vec![
                (
                    "",
                    Some((None, None, None, OptIndex::default(), None, None)),
                ),
                (
                    "o=b",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=b",
                    Some((None, None, Some(astr("b")), OptIndex::default(), None, None)),
                ),
                (
                    "=b!",
                    Some((
                        None,
                        None,
                        Some(astr("b")),
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@>1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "o=b@<8",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(8),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@>1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-o=b@<8",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(8),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@>1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "--o=b@<42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@>12",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(12),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "o=b!@<42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@>11",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(11),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "-o=b!@<4",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(4),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@<1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(1),
                        None,
                        Some(true),
                    )),
                ),
                (
                    "--o=b!@>42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@>1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "o=b/@<2",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(2),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@>1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "-o=b/@<42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@<11",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(11),
                        Some(true),
                        None,
                    )),
                ),
                (
                    "--o=b/@>42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@>1",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b!/@<42",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@>1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b!/@<42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@>11",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(11),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "o=b/!@<4",
                    Some((
                        None,
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(4),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@>1",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "-o=b/!@<42",
                    Some((
                        Some(astr("-")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@>1",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::greater(1),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "--o=b/!@<42",
                    Some((
                        Some(astr("--")),
                        Some(astr("o")),
                        Some(astr("b")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=bar",
                    Some((
                        None,
                        None,
                        Some(astr("bar")),
                        OptIndex::default(),
                        None,
                        None,
                    )),
                ),
                (
                    "=bar!",
                    Some((
                        None,
                        None,
                        Some(astr("bar")),
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@>1",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        OptIndex::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "option=bar@<42",
                    Some((
                        None,
                        Some(astr("option")),
                        Some(astr("bar")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@>1",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        OptIndex::greater(1),
                        None,
                        None,
                    )),
                ),
                (
                    "-option=bar@<42",
                    Some((
                        Some(astr("-")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@>11",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        OptIndex::greater(11),
                        None,
                        None,
                    )),
                ),
                (
                    "--option=bar@<42",
                    Some((
                        Some(astr("--")),
                        Some(astr("option")),
                        Some(astr("bar")),
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
            ];
            let parser = StrParser::default().with_pre("--").with_pre("-");

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
            OptIndex,
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
                assert_eq!(Some(&except.3), index.or(Some(&OptIndex::default())));
                assert_eq!(except.4, dk.deactivate);
                assert_eq!(except.5, dk.optional);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
