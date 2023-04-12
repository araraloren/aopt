use std::ops::RangeBounds;

pub use aopt::prelude::ErasedTy;
pub use aopt::prelude::ValValidator;

pub fn value<K, T>(val: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |inner_val| &val == inner_val)
}

pub fn array<const N: usize, K, T>(vals: [K; N]) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |val| vals.iter().any(|v| PartialEq::eq(v, val)))
}

pub fn vector<K, T>(vals: Vec<K>) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |val| vals.iter().any(|v| PartialEq::eq(v, val)))
}

pub fn range<K, T>(range: impl RangeBounds<K> + ErasedTy) -> ValValidator<T>
where
    T: ErasedTy + PartialOrd<K>,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| range.contains(val))
}

pub fn range_opt<K, T>(range: impl RangeBounds<K> + ErasedTy) -> ValValidator<Option<T>>
where
    T: ErasedTy + PartialOrd<K>,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val: &Option<T>| {
        if let Some(val) = val.as_ref() {
            range.contains(val)
        } else {
            false
        }
    })
}

pub fn greater<K, T>(start: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &start < val)
}

pub fn less<K, T>(end: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &end > val)
}

pub fn greater_or_eq<K, T>(start: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &start <= val)
}

pub fn less_or_eq<K, T>(end: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &end >= val)
}

#[cfg(feature = "sync")]
pub fn valid<T: ErasedTy>(func: impl Fn(&T) -> bool + Send + Sync + 'static) -> ValValidator<T> {
    ValValidator::from_fn(move |val| func(val))
}

#[cfg(not(feature = "sync"))]
pub fn valid<T: ErasedTy>(func: impl Fn(&T) -> bool + 'static) -> ValValidator<T> {
    ValValidator::from_fn(move |val| func(val))
}

#[macro_export]
macro_rules! valid {
    ($value:literal) => {
        $crate::valid::value($value)
    };

    ([$($value:literal),+]) => {
        $crate::valid::array([$($value),+])
    };

    (vec![$($value:literal),+]) => {
        $crate::valid::vector(vec![$($value),+])
    };

    ($start:literal .. $end:literal) => {
        $crate::valid::range($start .. $end)
    };

    ($start:literal ..) => {
        $crate::valid::range($start ..)
    };

    ($start:literal ..= $end:literal) => {
        $crate::valid::range($start ..= $end)
    };

    (> $value:literal) => {
        $crate::valid::greater($value)
    };

    (< $value:literal) => {
        $crate::valid::less($value)
    };

    (>= $value:literal) => {
        $crate::valid::greater_or_eq($value)
    };

    (>= $value:literal) => {
        $crate::valid::less_or_eq($value)
    };

    ($func:expr) => {
        $crate::valid::valid($func)
    };
}