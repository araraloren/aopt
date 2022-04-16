use ustr::Ustr;

use regex::Regex;

use crate::err::Error;
use crate::err::Result;
use crate::opt::OptIndex;
use crate::parser::PrefixedParser;

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
/// use aopt::set::OptConstructor;
/// use aopt::parser::PrefixedParser;
///
/// fn main() -> Result<()> {
///     let parser = OptConstructor::new(vec![gstr("--")])?;
///     let ret = parser.parse("--aopt=t!/".into())?;
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
///     let ret = parser.parse("bopt=t@[1,2,3]".into())?;
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
/// For more examples, please reference test case [`test_option_str_parser`](../../src/aopt/set/parser.rs.html#542).
///
#[derive(Debug)]
pub struct OptConstructor {
    regex: Regex,
    prefixs: Vec<Ustr>,
}

impl OptConstructor {
    pub fn new(prefixs: Vec<Ustr>) -> Result<Self> {
        Ok(Self {
            regex: Regex::new(r"^([^=]+)?(=([^=/!@]+))?([!/])?([!/])?(@(?:([+-><])?(\d+)|([+-])?(\[(?:\s*\d+,?\s*)+\])|(\*)))?$")
                .map_err(|e|Error::raise_error(format!("Can not initialize the argument regex!?: {:?}", e)))?,
            prefixs,
        })
    }

    pub fn get_regex(&self) -> &Regex {
        &self.regex
    }

    pub fn get_prefixs(&self) -> &[Ustr] {
        &self.prefixs
    }

    // the index number is small in generally
    fn parse_as_usize(pattern: &Ustr, data: &str) -> Result<usize> {
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
                    Error::opt_parsing_index_failed(
                        pattern.to_string(),
                        format!("{:?} is not a valid number", data),
                    )
                })? as usize;
        }
        if count == 0 {
            return Err(Error::opt_parsing_index_failed(
                pattern.to_string(),
                format!("{:?} is not a valid number", data),
            ));
        }
        Ok(ret)
    }

    fn parse_as_usize_sequence(pattern: &Ustr, data: &str) -> Result<Vec<usize>> {
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
                    return Err(Error::opt_parsing_index_failed(
                        pattern.to_string(),
                        format!("{:?} is not a valid number sequence", data),
                    ));
                }
                ret.push(Self::parse_as_usize(pattern, &data[last..index])?);
                last = index + 1;
            }
        }
        Ok(ret)
    }

    pub fn parse_creator_string(&self, pattern: Ustr, prefix: Ustr) -> Result<DataKeeper> {
        let (_, left_part) = pattern.split_at(prefix.len());

        if let Some(cap) = self.get_regex().captures(left_part) {
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
            Ok(DataKeeper {
                pattern,
                name: cap.get(IDX_NAME).map(|v| Ustr::from(v.as_str())),
                prefix: Some(prefix),
                deactivate,
                optional,
                type_name: cap.get(IDX_TYPE).map(|v| Ustr::from(v.as_str())),
                forward_index,
                backward_index,
                anywhere,
                list,
                except,
                greater,
                less,
            })
        } else {
            Err(Error::opt_parsing_constructor_failed(pattern))
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

impl PrefixedParser for OptConstructor {
    type Output = DataKeeper;

    fn parse(&self, pattern: Ustr) -> Result<Self::Output> {
        if !pattern.is_empty() {
            for prefix in self.get_prefixs() {
                if pattern.starts_with(prefix.as_str()) {
                    if let Ok(data_keeper) = self.parse_creator_string(pattern, *prefix) {
                        return Ok(data_keeper);
                    }
                }
            }
            if let Ok(mut data_keeper) = self.parse_creator_string(pattern, Ustr::from("")) {
                data_keeper.prefix = None;
                return Ok(data_keeper);
            }
        }
        Err(Error::opt_parsing_constructor_failed(pattern))
    }

    fn get_prefixs(&self) -> &[Ustr] {
        &self.prefixs
    }
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
    pub fn gen_index(&mut self) -> OptIndex {
        if self.forward_index.is_some() {
            OptIndex::forward(self.forward_index.unwrap())
        } else if self.backward_index.is_some() {
            OptIndex::backward(self.backward_index.unwrap())
        } else if self.anywhere.unwrap_or(false) {
            OptIndex::anywhere()
        } else if !self.list.is_empty() {
            OptIndex::list(std::mem::take(&mut self.list))
        } else if !self.except.is_empty() {
            OptIndex::except(std::mem::take(&mut self.except))
        } else if self.greater.is_some() {
            OptIndex::greater(self.greater.unwrap())
        } else if self.less.is_some() {
            OptIndex::less(self.less.unwrap())
        } else {
            OptIndex::default()
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

#[cfg(test)]
mod test {
    use super::OptConstructor;
    use crate::gstr;
    use crate::opt::OptIndex;
    use crate::parser::PrefixedParser;
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
                        Some(true),
                        Some(true),
                    )),
                ),
                (
                    "=b",
                    Some((None, None, Some(gstr("b")), OptIndex::default(), None, None)),
                ),
                (
                    "=b!",
                    Some((
                        None,
                        None,
                        Some(gstr("b")),
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
                        Some(gstr("b")),
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
                        Some(gstr("b")),
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
                        Some(gstr("b")),
                        OptIndex::default(),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(8),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(8),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::anywhere(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::greater(12),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(11),
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
                        OptIndex::less(4),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::less(1),
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
                        OptIndex::greater(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(2),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::less(11),
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
                        OptIndex::greater(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(11),
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
                        OptIndex::less(4),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        OptIndex::default(),
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
                        Some(gstr("bar")),
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
                        Some(gstr("bar")),
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
                        Some(gstr("bar")),
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
                        Some(gstr("bar")),
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
                        Some(gstr("bar")),
                        OptIndex::default(),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(1),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::anywhere(),
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
                        OptIndex::greater(11),
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
                        OptIndex::less(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
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
                        OptIndex::forward(1),
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
                        OptIndex::backward(1),
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
                        OptIndex::forward(42),
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
                        OptIndex::list(vec![1, 2, 3]),
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
                        OptIndex::list(vec![4, 5, 12]),
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
                        OptIndex::except(vec![1, 2, 4]),
                        Some(true),
                        Some(true),
                    )),
                ),
            ];

            let parser = OptConstructor::new(vec![gstr("--"), gstr("-")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &parser, &case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Ustr,
        parser: &OptConstructor,
        except: &Option<(
            Option<Ustr>,
            Option<Ustr>,
            Option<Ustr>,
            OptIndex,
            Option<bool>,
            Option<bool>,
        )>,
    ) {
        let ret = parser.parse(pattern);

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
