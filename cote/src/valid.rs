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

pub fn value_opt<K, T>(val: K) -> ValValidator<Option<T>>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |inner_val: &Option<T>| {
        inner_val
            .as_ref()
            .map(|inner_val| &val == inner_val)
            .unwrap_or_default()
    })
}

pub fn array<const N: usize, K, T>(vals: [K; N]) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |val| vals.iter().any(|v| PartialEq::eq(v, val)))
}

pub fn array_opt<const N: usize, K, T>(vals: [K; N]) -> ValValidator<Option<T>>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |val: &Option<T>| {
        val.as_ref()
            .map(|val| vals.iter().any(|v| PartialEq::eq(v, val)))
            .unwrap_or_default()
    })
}

pub fn vector<K, T>(vals: Vec<K>) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |val| vals.iter().any(|v| PartialEq::eq(v, val)))
}

pub fn vector_opt<K, T>(vals: Vec<K>) -> ValValidator<Option<T>>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |val: &Option<T>| {
        val.as_ref()
            .map(|val| vals.iter().any(|v| PartialEq::eq(v, val)))
            .unwrap_or_default()
    })
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
        val.as_ref()
            .map(|val| range.contains(val))
            .unwrap_or_default()
    })
}

pub fn greater<K, T>(start: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &start < val)
}

pub fn greater_opt<K, T>(start: K) -> ValValidator<Option<T>>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val: &Option<T>| {
        val.as_ref().map(|v| &start < v).unwrap_or_default()
    })
}

pub fn less<K, T>(end: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &end > val)
}

pub fn less_opt<K, T>(end: K) -> ValValidator<Option<T>>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val: &Option<T>| {
        val.as_ref().map(|val| &end > val).unwrap_or_default()
    })
}

pub fn greater_or_eq<K, T>(start: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &start <= val)
}

pub fn greater_or_eq_opt<K, T>(start: K) -> ValValidator<Option<T>>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val: &Option<T>| {
        val.as_ref().map(|val| &start <= val).unwrap_or_default()
    })
}

pub fn less_or_eq<K, T>(end: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &end >= val)
}

pub fn less_or_eq_opt<K, T>(end: K) -> ValValidator<Option<T>>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val: &Option<T>| {
        val.as_ref().map(|val| &end >= val).unwrap_or_default()
    })
}

#[cfg(feature = "sync")]
pub fn valid_fn<T: ErasedTy>(func: impl Fn(&T) -> bool + Send + Sync + 'static) -> ValValidator<T> {
    ValValidator::from_fn(move |val| func(val))
}

#[cfg(feature = "sync")]
pub fn valid_opt_fn<T: ErasedTy>(
    func: impl Fn(&Option<T>) -> bool + Send + Sync + 'static,
) -> ValValidator<Option<T>> {
    ValValidator::from_fn(move |val: &Option<T>| func(val))
}

#[cfg(not(feature = "sync"))]
pub fn valid_fn<T: ErasedTy>(func: impl Fn(&T) -> bool + 'static) -> ValValidator<T> {
    ValValidator::from_fn(move |val| func(val))
}

#[cfg(not(feature = "sync"))]
pub fn valid_opt_fn<T: ErasedTy>(
    func: impl Fn(&Option<T>) -> bool + 'static,
) -> ValValidator<Option<T>> {
    ValValidator::from_fn(move |val: &Option<T>| func(val))
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
        $crate::valid::valid_fn($func)
    };
}

#[macro_export]
macro_rules! valid_opt {
    ($value:literal) => {
        $crate::valid::value_opt($value)
    };

    ([$($value:literal),+]) => {
        $crate::valid::array_opt([$($value),+])
    };

    (vec![$($value:literal),+]) => {
        $crate::valid::vector_opt(vec![$($value),+])
    };

    ($start:literal .. $end:literal) => {
        $crate::valid::range_opt($start .. $end)
    };

    ($start:literal ..) => {
        $crate::valid::range_opt($start ..)
    };

    ($start:literal ..= $end:literal) => {
        $crate::valid::range_opt($start ..= $end)
    };

    (> $value:literal) => {
        $crate::valid::greater_opt($value)
    };

    (< $value:literal) => {
        $crate::valid::less_opt($value)
    };

    (>= $value:literal) => {
        $crate::valid::greater_or_eq_opt($value)
    };

    (>= $value:literal) => {
        $crate::valid::less_or_eq_opt($value)
    };

    ($func:expr) => {
        $crate::valid::valid_opt_fn($func)
    };
}
