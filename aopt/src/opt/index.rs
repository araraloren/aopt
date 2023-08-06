use std::cell::Ref;
use std::cell::RefCell;
use std::ops::Range;
use std::ops::RangeBounds;
use std::ops::RangeFrom;
use std::ops::RangeFull;
use std::ops::RangeInclusive;
use std::ops::RangeTo;
use std::ops::RangeToInclusive;

use neure::neure;
use neure::regex;
use neure::CharsCtx;
use neure::Context;

use crate::raise_error;
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

thread_local! {
    static IDX_PARSER: RefCell<CharsCtx> = RefCell::new(CharsCtx::new("", KEY_TOTAL));
}

const KEY_ANYWHERE: usize = 0;
const KEY_PLUS: usize = 1;
const KEY_MINUS: usize = 2;
const KEY_SEQ: usize = 3;
const KEY_START: usize = 4;
const KEY_END: usize = 5;
const KEY_RANGE: usize = 6;
const KEY_TOTAL: usize = 7;

impl Index {
    // the index number is small in generally
    pub(crate) fn parse_as_usize(pattern: &str, data: &str) -> Result<usize, Error> {
        data.parse::<usize>().map_err(|e| {
            Error::invalid_opt_index(pattern, "invalid usize number").cause_by(e.into())
        })
    }

    #[inline(always)]
    pub fn parse_ctx(ctx: &mut CharsCtx) -> Result<(), neure::err::Error> {
        let start = neure::start();
        let end = neure::end();
        let anywhere = neure!('*');
        let plus = neure!('+');
        let minus = neure!('-');
        let left = neure!('[');
        let right = neure!(']');
        let digit = neure!(['0' - '9']+);
        let comma = neure!(',');
        let range_op = neure!('.'{2});
        let space = neure!(*);
        let whole_parser = move |ctx: &mut CharsCtx| -> Result<(), neure::err::Error> {
            ctx.try_mat(&start)?;
            if !ctx.cap(KEY_ANYWHERE, &anywhere) {
                let _ = ctx.cap(KEY_PLUS, &plus) || ctx.cap(KEY_MINUS, &minus);

                if ctx.mat(&left) {
                    let ret = ctx.try_cap(KEY_SEQ, &digit)? > 0 && ctx.mat(&comma);

                    ctx.mat(&space);
                    if ret {
                        while ctx.cap(KEY_SEQ, &digit) && ctx.mat(&comma) {}
                    }
                    ctx.try_mat(&right)?;
                } else {
                    ctx.cap(KEY_START, &digit); // start may not exist
                    ctx.mat(&space);
                    if ctx.cap(KEY_RANGE, &range_op) {
                        ctx.mat(&space);
                        if ctx.contain(KEY_START) {
                            ctx.cap(KEY_END, &digit);
                        } else {
                            ctx.try_cap(KEY_END, &digit)?;
                        }
                    }
                }
            }
            ctx.try_mat(&end)?;
            Ok(())
        };

        whole_parser(ctx)
    }

    pub fn substr_of_ctx<'a>(
        ctx: &'a Ref<'a, CharsCtx>,
        span_id: usize,
        index: usize,
    ) -> Result<&'a str, Error> {
        let spans = ctx.spans(span_id).ok_or_else(|| {
            raise_error!("Can not get span data for `{}` from `{:?}`", span_id, ctx)
        })?;
        let span = spans
            .get(index)
            .ok_or_else(|| raise_error!("Invalid span data from `{:?}`", ctx))?;
        ctx.substr(span)
            .map_err(|e| raise_error!("Can not get substr from `{:?}`: {:?}", ctx, e))
    }

    pub fn parse(pat: &str) -> Result<Self, Error> {
        IDX_PARSER
            .try_with(|ctx| {
                if Self::parse_ctx(ctx.borrow_mut().reset_with(pat)).is_ok() {
                    let ctx = ctx.borrow();

                    if ctx.contain(KEY_ANYWHERE) {
                        Ok(Self::anywhere())
                    } else if ctx.contain(KEY_RANGE) {
                        let range_beg = Self::substr_of_ctx(&ctx, KEY_START, 0).ok();
                        let range_end = Self::substr_of_ctx(&ctx, KEY_END, 0).ok();

                        match (range_beg, range_end) {
                            (None, None) => {
                                return Err(Error::invalid_opt_index(pat, "index can not be empty"))
                            }
                            (None, Some(end)) => {
                                Ok(Self::range(None, Some(Self::parse_as_usize(pat, end)?)))
                            }
                            (Some(beg), None) => {
                                Ok(Self::range(Some(Self::parse_as_usize(pat, beg)?), None))
                            }
                            (Some(beg), Some(end)) => {
                                let beg = Self::parse_as_usize(pat, beg)?;
                                let end = Self::parse_as_usize(pat, end)?;

                                if beg <= end {
                                    Ok(Self::range(Some(beg), Some(end)))
                                } else {
                                    return Err(Error::invalid_opt_index(
                                        pat,
                                        "assert failed on (beg <= end)",
                                    ));
                                }
                            }
                        }
                    } else if let Some(value) = ctx.spans(KEY_SEQ) {
                        let mut list = vec![];

                        for span in value {
                            list.push(Self::parse_as_usize(
                                pat,
                                ctx.substr(span).map_err(|e| {
                                    raise_error!("Can not get substr from `{:?}`: {:?}", ctx, e)
                                })?,
                            )?);
                        }
                        if ctx.contain(KEY_MINUS) {
                            Ok(Self::except(list))
                        } else {
                            Ok(Self::list(list))
                        }
                    } else if ctx.contain(KEY_START) {
                        let index =
                            Self::parse_as_usize(pat, Self::substr_of_ctx(&ctx, KEY_START, 0)?)?;

                        if ctx.contain(KEY_MINUS) {
                            Ok(Self::backward(index))
                        } else {
                            Ok(Self::forward(index))
                        }
                    } else {
                        Err(Error::invalid_opt_index(pat, "unknown index create string"))
                    }
                } else {
                    Err(Error::invalid_opt_index(
                        pat,
                        "index create string parsing failed",
                    ))
                }
            })
            .map_err(|e| {
                Error::local_access("can not access index parsing regex").cause_by(e.into())
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
                    Err(raise_error!(
                        "start position of Index can't be negative: {:?}",
                        range.start_bound()
                    ))
                } else {
                    Ok(Self::range(Some(*s - 1), Some(e + 1)))
                }
            }
            (std::ops::Bound::Excluded(s), std::ops::Bound::Excluded(e)) => {
                if *s == 0 {
                    Err(raise_error!(
                        "start position of Index can't be negative: {:?}",
                        range.start_bound()
                    ))
                } else {
                    Ok(Self::range(Some(*s - 1), Some(*e)))
                }
            }
            (std::ops::Bound::Excluded(s), std::ops::Bound::Unbounded) => {
                if *s == 0 {
                    Err(raise_error!(
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

impl TryFrom<crate::Str> for Index {
    type Error = Error;

    fn try_from(value: crate::Str) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
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
