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
    ValValidator::from_fn(move |val| vals.iter().any(|v| PartialEq::eq(v, &val)))
}

pub fn vector<K, T>(vals: Vec<K>) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    ValValidator::from_fn(move |val| vals.iter().any(|v| PartialEq::eq(v, &val)))
}

pub fn range<K, T>(start: K, end: K) -> ValValidator<T>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    ValValidator::from_fn(move |val| &start <= val && &end > val)
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
