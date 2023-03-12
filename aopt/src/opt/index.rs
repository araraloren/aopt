use regex::Regex;

use crate::Error;

/// Index using for option match.
///
/// The index is the position of left arguments (non-option arguments, NOA) index.
///
/// # Example
///
/// ```txt
/// foo.exe -a=value -b value pos1 --aopt=42 pos2 --bopt value pos3
///             |     |   |    |      |       |      |     |     |
///             |     |   |    |      |       |      |     |     NOA @3 or @-1
///             |     |   |    |      |       |      |     value of --bopt
///             |     |   |    |      |       |      option --bopt
///             |     |   |    |      |       NOA @2 or @-2
///             |     |   |    |    option --aopt and its value 42
///             |     |   |   NOA @1
///             |     |   value of -b
///             |    option -b
///         option -a and its value
/// ```
///
/// For option check, see [`SetChecker`](crate::set::SetChecker) for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Index {
    /// The forward index of NOA, fixed position.
    ///
    /// # Example
    ///
    /// For `["app", "--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@1` will matching `"pos1"`.
    ///
    /// `@2` will matching `"pos2"`.
    ///
    /// `@3` will matching `"pos3"`.
    Forward(usize),

    /// The backward index of NOA, floating position.
    ///
    /// # Example
    ///
    /// For `["app", "--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@-1` will matching `"pos2"`.
    ///
    /// `@-2` will matching `"pos1"`.
    ///
    /// `@-3` will matching `"app"`.
    Backward(usize),

    /// The include list of forward index of NOA, fixed position.
    ///
    /// # Example
    ///
    /// For `["app", "--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@[1,3]` will matching `"pos1"` or `"pos3"`.
    ///
    /// `@[1,2]` will matching `"pos1"` or `"pos2"`.
    ///
    /// `@[1,2,3]` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    List(Vec<usize>),

    /// The exclude list of forward index of NOA, floating position.
    ///
    /// # Example
    ///
    /// For `["app", "--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@-[1,3]` will matching `"pos2"`.
    ///
    /// `@-[3]` will matching `"pos1"` or `"pos2"`.
    ///
    /// `@-[2]` will matching `"pos1"` or `"pos3"`.
    Except(Vec<usize>),

    /// The NOA which index inside in given position range with format `(m..n]`.
    ///
    /// If range have upper limit, the index is fixed position otherwise it is floating position.
    ///
    /// # Example
    ///
    /// For `["app", "--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@0..` will matching `"app"`, `"pos1"`, `"pos2"` or `"pos3"`.
    ///
    /// `@2..` will matching `"pos2"`, `"pos3"`.
    ///
    /// `@1..` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    ///
    /// `@..4` will matching `"app"`, `"pos1"`, `"pos2"` or `"pos3"`.
    ///
    /// `@..2` will matching `"app"`, `"pos1"`.
    ///
    /// `@1..3` will matching `"pos1"`, `"pos2"`.
    Range(usize, Option<usize>),

    /// The anywhere position of NOA, floating position.
    ///
    /// # Example
    ///
    /// For `["app", "--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@*` will matching `"app"`, `"pos1"`, `"pos2"` or `"pos3"`.
    AnyWhere,

    Null,
}

thread_local! {
    static IDX_PARSER: Regex = Regex::new(r"^(?:([+-])?(\d+)|(\d+)?(..)(\d+)?|([+-])?(\[(?:\s*\d+,?\s*)+\])|(\*))$").unwrap();
}

const IDX_INDEX: usize = 2;
const IDX_INDEX_SIGN: usize = 1;
const IDX_RANGE: usize = 4;
const IDX_RANGE_BEG: usize = 3;
const IDX_RANGE_END: usize = 5;
const IDX_SEQUENCE: usize = 7;
const IDX_SEQUENCE_SIGN: usize = 6;
const IDX_ANYWHERE: usize = 8;

impl Index {
    // the index number is small in generally
    pub(crate) fn parse_as_usize(pattern: &str, data: &str) -> Result<usize, Error> {
        data.parse::<usize>().map_err(|e| {
            Error::invalid_opt_index(pattern, &format!("invalid usize number: {:?}", e))
        })
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
                    return Err(Error::invalid_opt_index(
                        pattern,
                        &format!("{} not a valid usize number", data),
                    ));
                }
                ret.push(Self::parse_as_usize(pattern, &data[last..index])?);
                last = index + 1;
            }
        }
        Ok(ret)
    }

    pub fn parse(pattern: &str) -> Result<Self, Error> {
        IDX_PARSER
            .try_with(|regex| {
                if let Some(cap) = regex.captures(pattern) {
                    if let Some(value) = cap.get(IDX_INDEX) {
                        let index = Self::parse_as_usize(pattern, value.as_str())?;
                        let sign = cap
                            .get(IDX_INDEX_SIGN)
                            .map(|sign| sign.as_str() == "-")
                            .unwrap_or(false);

                        if sign {
                            Ok(Self::backward(index))
                        } else {
                            Ok(Self::forward(index))
                        }
                    } else if cap.get(IDX_RANGE).is_some() {
                        let range_beg = cap.get(IDX_RANGE_BEG);
                        let range_end = cap.get(IDX_RANGE_END);

                        match (range_beg, range_end) {
                            (None, None) => {
                                return Err(Error::invalid_opt_index(
                                    pattern,
                                    "index can not be empty",
                                ))
                            }
                            (None, Some(end)) => Ok(Self::range(
                                None,
                                Some(Self::parse_as_usize(pattern, end.as_str())?),
                            )),
                            (Some(beg), None) => Ok(Self::range(
                                Some(Self::parse_as_usize(pattern, beg.as_str())?),
                                None,
                            )),
                            (Some(beg), Some(end)) => {
                                let beg = Self::parse_as_usize(pattern, beg.as_str())?;
                                let end = Self::parse_as_usize(pattern, end.as_str())?;

                                if beg <= end {
                                    Ok(Self::range(Some(beg), Some(end)))
                                } else {
                                    return Err(Error::invalid_opt_index(
                                        pattern,
                                        "assert failed on (beg <= end)",
                                    ));
                                }
                            }
                        }
                    } else if let Some(value) = cap.get(IDX_SEQUENCE) {
                        let list = Self::parse_as_usize_sequence(pattern, value.as_str())?;
                        let sign = cap
                            .get(IDX_SEQUENCE_SIGN)
                            .map(|sign| sign.as_str() == "-")
                            .unwrap_or(false);

                        if sign {
                            Ok(Self::except(list))
                        } else {
                            Ok(Self::list(list))
                        }
                    } else if cap.get(IDX_ANYWHERE).is_some() {
                        Ok(Self::anywhere())
                    } else {
                        Err(Error::invalid_opt_index(
                            pattern,
                            "invalid index create string",
                        ))
                    }
                } else {
                    Err(Error::invalid_opt_index(
                        pattern,
                        "index create string parsing failed",
                    ))
                }
            })
            .map_err(|e| {
                Error::raise_error(format!("can not access index parsing regex: {:?}", e))
            })?
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_forward(&self) -> bool {
        matches!(self, Self::Forward(_))
    }

    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub fn is_except(&self) -> bool {
        matches!(self, Self::Except(_))
    }

    pub fn is_range(&self) -> bool {
        matches!(self, Self::Range(_, _))
    }

    pub fn is_anywhere(&self) -> bool {
        matches!(self, Self::AnyWhere)
    }

    pub fn to_help(&self) -> String {
        String::default()
    }

    pub fn forward(index: usize) -> Self {
        Self::Forward(index)
    }

    pub fn backward(index: usize) -> Self {
        Self::Backward(index)
    }

    pub fn list(list: Vec<usize>) -> Self {
        Self::List(list)
    }

    pub fn except(list: Vec<usize>) -> Self {
        Self::Except(list)
    }

    pub fn range(start: Option<usize>, end: Option<usize>) -> Self {
        match (start, end) {
            (None, None) => {
                panic!("start and end can't both None")
            }
            (None, _) => Self::Range(0, end),
            (Some(start), _) => Self::Range(start, end),
        }
    }

    pub fn anywhere() -> Self {
        Self::AnyWhere
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn calc_index(&self, noa_index: usize, noa_count: usize) -> Option<usize> {
        match self {
            Self::Forward(offset) => {
                let offset = *offset;

                if offset < noa_count {
                    return Some(offset);
                }
            }
            Self::Backward(offset) => {
                let offset = *offset;

                if offset < noa_count {
                    return Some(noa_count - offset - 1);
                }
            }
            Self::List(list) => {
                for offset in list {
                    let offset = *offset;

                    if offset < noa_count && offset == noa_index {
                        return Some(offset);
                    }
                }
            }
            Self::Except(list) => {
                if noa_index < noa_count && !list.contains(&noa_index) {
                    return Some(noa_index);
                }
            }
            Self::Range(start, end) => match (start, end) {
                (start, None) => {
                    let start = *start;

                    if noa_index >= start {
                        return Some(noa_index);
                    }
                }
                (start, Some(end)) => {
                    let start = *start;
                    let end = *end;

                    if noa_index >= start && noa_index < end {
                        return Some(noa_index);
                    }
                }
            },
            Self::AnyWhere => {
                return Some(noa_index);
            }
            _ => {}
        }
        None
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::Null
    }
}

impl ToString for Index {
    fn to_string(&self) -> String {
        match self {
            Index::AnyWhere => "*".to_string(),
            Index::Null => String::default(),
            Index::Forward(v) => {
                format!("{}", v)
            }
            Index::Backward(v) => {
                format!("-{}", v)
            }
            Index::Range(s, None) => {
                format!("{}..", s,)
            }
            Index::Range(s, Some(e)) => {
                format!("{}..{}", s, e)
            }
            Index::List(v) => {
                let strs: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();

                format!("[{}]", strs.join(", "))
            }
            Index::Except(v) => {
                let strs: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();

                format!("-[{}]", strs.join(", "))
            }
        }
    }
}
