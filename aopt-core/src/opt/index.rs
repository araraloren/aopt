use std::fmt::Display;
use std::ops::Range;
use std::ops::RangeBounds;
use std::ops::RangeFrom;
use std::ops::RangeFull;
use std::ops::RangeInclusive;
use std::ops::RangeTo;
use std::ops::RangeToInclusive;

use crate::err::Error;
use crate::error;

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
/// For option check, see [`SetChecker`](https://docs.rs/aopt/latest/aopt/set/trait.SetChecker.html) for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
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

    #[default]
    Null,
}

impl Index {
    pub fn parse(dat: &str) -> Result<Self, Error> {
        use neure::prelude::*;

        let pos = "+".opt();
        let neg = "-";
        let num = char::is_ascii_digit.many1();
        let num = num.try_map(map::from_str::<usize>());

        let anywhere = "*".map(|_| Index::anywhere());

        let num_vector = num.sep(",").enclose("[", "]");
        let except = num_vector.prefix(neg).map(Index::except);
        let list = num_vector.prefix(pos).map(Index::list);

        let range = num
            .opt()
            .sep_once("..", num.opt())
            .map(|(beg, end)| Index::range(beg, end));

        let backward = num.prefix(neg).map(Index::backward);
        let forward = num.prefix(pos).map(Index::forward);

        let parser = anywhere
            .or(except)
            .or(list)
            .or(range)
            .or(backward)
            .or(forward)
            .suffix(regex::end())
            .prefix(regex::start());

        CharsCtx::new(dat)
            .skip_ascii_whitespace()
            .ctor(&parser)
            .map_err(|_| Error::index_parse(dat, "failed parsing index"))
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
        match self {
            Index::Forward(offset) => {
                format!("{}", offset)
            }
            Index::Backward(offset) => {
                format!("-{}", offset)
            }
            Index::List(list) => {
                format!(
                    "[{}]",
                    list.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Index::Except(list) => {
                format!(
                    "-[{}]",
                    list.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Index::Range(start, None) => {
                format!("{}..", start)
            }
            Index::Range(start, Some(end)) => {
                format!("{}..{}", start, end)
            }
            Index::AnyWhere => "*".to_string(),
            Index::Null => String::default(),
        }
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

    pub(crate) fn from_range(range: &impl RangeBounds<usize>) -> Result<Self, Error> {
        match (range.start_bound(), range.end_bound()) {
            (std::ops::Bound::Included(s), std::ops::Bound::Included(e)) => {
                Ok(Self::range(Some(*s), Some(e + 1)))
            }
            (std::ops::Bound::Included(s), std::ops::Bound::Excluded(e)) => {
                Ok(Self::range(Some(*s), Some(*e)))
            }
            (std::ops::Bound::Included(s), std::ops::Bound::Unbounded) => {
                Ok(Self::range(Some(*s), None))
            }
            (std::ops::Bound::Excluded(s), std::ops::Bound::Included(e)) => {
                if *s == 0 {
                    Err(error!(
                        "start position of Index can't be negative: {:?}",
                        range.start_bound()
                    ))
                } else {
                    Ok(Self::range(Some(*s - 1), Some(e + 1)))
                }
            }
            (std::ops::Bound::Excluded(s), std::ops::Bound::Excluded(e)) => {
                if *s == 0 {
                    Err(error!(
                        "start position of Index can't be negative: {:?}",
                        range.start_bound()
                    ))
                } else {
                    Ok(Self::range(Some(*s - 1), Some(*e)))
                }
            }
            (std::ops::Bound::Excluded(s), std::ops::Bound::Unbounded) => {
                if *s == 0 {
                    Err(error!(
                        "start position of Index can't be negative: {:?}",
                        range.start_bound()
                    ))
                } else {
                    Ok(Self::range(Some(*s - 1), None))
                }
            }
            (std::ops::Bound::Unbounded, std::ops::Bound::Included(e)) => {
                Ok(Self::range(Some(0), Some(*e - 1)))
            }
            (std::ops::Bound::Unbounded, std::ops::Bound::Excluded(e)) => {
                Ok(Self::range(Some(0), Some(*e)))
            }
            (std::ops::Bound::Unbounded, std::ops::Bound::Unbounded) => {
                panic!("start and end of Index can't both unbounded")
            }
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

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
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
        )
    }
}

impl TryFrom<String> for Index {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl<'a> TryFrom<&'a str> for Index {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

macro_rules! impl_range_for {
    ($range:ty) => {
        impl TryFrom<$range> for Index {
            type Error = Error;

            fn try_from(value: $range) -> Result<Self, Self::Error> {
                Self::from_range(&value)
            }
        }

        impl<'a> TryFrom<&'a $range> for Index {
            type Error = Error;

            fn try_from(value: &'a $range) -> Result<Self, Self::Error> {
                Self::from_range(value)
            }
        }
    };
}

impl_range_for!(Range<usize>);

impl_range_for!(RangeFrom<usize>);

impl_range_for!(RangeInclusive<usize>);

impl_range_for!(RangeTo<usize>);

impl_range_for!(RangeToInclusive<usize>);

impl_range_for!(RangeFull);

macro_rules! impl_signed_ty_for {
    ($int:ty) => {
        impl TryFrom<$int> for Index {
            type Error = Error;

            fn try_from(value: $int) -> Result<Self, Self::Error> {
                Ok(if value >= 0 {
                    Self::forward(value as usize)
                } else {
                    Self::backward((-value) as usize)
                })
            }
        }
    };
}

impl_signed_ty_for!(isize);

impl_signed_ty_for!(i128);

impl_signed_ty_for!(i64);

impl_signed_ty_for!(i32);

impl_signed_ty_for!(i16);

impl_signed_ty_for!(i8);

macro_rules! impl_unsigned_ty_for {
    ($int:ty) => {
        impl TryFrom<$int> for Index {
            type Error = Error;

            fn try_from(value: $int) -> Result<Self, Self::Error> {
                Ok(Self::forward(value as usize))
            }
        }
    };
}

impl_unsigned_ty_for!(usize);

impl_unsigned_ty_for!(u128);

impl_unsigned_ty_for!(u64);

impl_unsigned_ty_for!(u32);

impl_unsigned_ty_for!(u16);

impl_unsigned_ty_for!(u8);

impl TryFrom<Vec<usize>> for Index {
    type Error = Error;

    fn try_from(value: Vec<usize>) -> Result<Self, Self::Error> {
        Ok(Self::list(value))
    }
}

impl<const N: usize> TryFrom<[usize; N]> for Index {
    type Error = Error;

    fn try_from(value: [usize; N]) -> Result<Self, Self::Error> {
        Ok(Self::list(Vec::from(value)))
    }
}
