use std::{marker::PhantomData, ops::RangeBounds};

pub use aopt::prelude::ErasedTy;
use aopt::value::{ValValidator, ValidatorHandler};

pub trait Validate<T>
where
    T: ErasedTy,
{
    fn check(&self, value: &T) -> bool;
}

pub struct Value<K>(K);

impl<K> Value<K> {
    pub fn new(value: K) -> Self {
        Self(value)
    }
}

impl<T, K> Validate<T> for Value<K>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    fn check(&self, value: &T) -> bool {
        &self.0 == value
    }
}

pub struct GreaterEqual<K>(K);

impl<K> GreaterEqual<K> {
    pub fn new(value: K) -> Self {
        Self(value)
    }
}

impl<T, K> Validate<T> for GreaterEqual<K>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    fn check(&self, value: &T) -> bool {
        &self.0 >= value
    }
}

pub struct LessEqual<K>(K);

impl<K> LessEqual<K> {
    pub fn new(value: K) -> Self {
        Self(value)
    }
}

impl<T, K> Validate<T> for LessEqual<K>
where
    T: ErasedTy,
    K: ErasedTy + PartialOrd<T>,
{
    fn check(&self, value: &T) -> bool {
        &self.0 <= value
    }
}

pub struct Array<const N: usize, K>([K; N]);

impl<const N: usize, K> Array<N, K> {
    pub fn new(value: [K; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize, T, K> Validate<T> for Array<N, K>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    fn check(&self, value: &T) -> bool {
        self.0.iter().any(|v| PartialEq::eq(v, value))
    }
}

pub struct Vector<K>(Vec<K>);

impl<K> Vector<K> {
    pub fn new(value: Vec<K>) -> Self {
        Self(value)
    }
}

impl<T, K> Validate<T> for Vector<K>
where
    T: ErasedTy,
    K: ErasedTy + PartialEq<T>,
{
    fn check(&self, value: &T) -> bool {
        self.0.iter().any(|v| PartialEq::eq(v, value))
    }
}

pub struct Range<K, R>(R, PhantomData<K>);

impl<K, R> Range<K, R>
where
    R: RangeBounds<K> + ErasedTy,
{
    pub fn new(value: R) -> Self {
        Self(value, PhantomData::default())
    }
}

impl<T, K, R> Validate<T> for Range<K, R>
where
    T: ErasedTy + PartialOrd<K>,
    K: ErasedTy + PartialOrd<T>,
    R: RangeBounds<K> + ErasedTy,
{
    fn check(&self, value: &T) -> bool {
        self.0.contains(value)
    }
}

pub struct Validator<T>(ValidatorHandler<T>);

impl<T> Validator<T>
where
    T: ErasedTy,
{
    #[cfg(feature = "sync")]
    pub fn new(func: impl Fn(&T) -> bool + Send + Sync + 'static) -> Self {
        Self(Box::new(move |val| func(val)))
    }

    #[cfg(not(feature = "sync"))]
    pub fn new(func: impl Fn(&T) -> bool + 'static) -> Self {
        Self(Box::new(move |val| func(val)))
    }
}

impl<T> Validate<T> for Validator<T>
where
    T: ErasedTy,
{
    fn check(&self, value: &T) -> bool {
        (self.0)(value)
    }
}

impl<T> From<Validator<T>> for ValValidator<T>
where
    T: ErasedTy,
{
    fn from(value: Validator<T>) -> Self {
        ValValidator::new(value.0)
    }
}

/// Check the value of option.
///
/// # Example
/// ```rust
/// # use cote::prelude::*;
/// # use cote::valid;
/// #
/// #[derive(Debug, Cote, PartialEq, Eq)]
/// #[cote(help)]
/// pub struct Cli {
///     #[arg(alias = "-v", valid = valid!(42))]
///     value: u64,
/// }
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///    
///     assert!(Cli::parse(Args::from_array(["app", "-v41"])).is_err());
///
///     assert!(Cli::parse(Args::from_array(["app", "-v42"])).is_ok());
///
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! valid {
    ($value:literal) => {
        $crate::valid::Value::new($value)
    };

    ([$($value:literal),+]) => {
        $crate::valid::Array::new([$($value),+])
    };

    (vec![$($value:literal),+]) => {
        $crate::valid::Vector::new(vec![$($value),+])
    };

    ($start:literal .. $end:literal) => {
        $crate::valid::Range::new($start .. $end)
    };

    ($start:literal ..) => {
        $crate::valid::Range::new($start ..)
    };

    ($start:literal ..= $end:literal) => {
        $crate::valid::Range::new($start ..= $end)
    };

    (.. $end:literal) => {
        $crate::valid::Range::new($start .. $end)
    };

    (..= $end:literal) => {
        $crate::valid::Range::new($start ..= $end)
    };

    (> $value:literal) => {
        $crate::valid::Range::new($value ..)
    };

    (< $value:literal) => {
        $crate::valid::Range::new(.. $value)
    };

    (>= $value:literal) => {
        $crate::valid::GreaterEqual::new($value)
    };

    (<= $value:literal) => {
        $crate::valid::Range::new(..= $value)
    };

    ($func:expr) => {
        $crate::valid::Validator::new($func)
    };
}
