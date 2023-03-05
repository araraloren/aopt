use crate::map::ErasedTy;

#[cfg(feature = "sync")]
pub type ValidatorHandler<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

#[cfg(not(feature = "sync"))]
pub type ValidatorHandler<T> = Box<dyn Fn(&T) -> bool>;

pub struct ValValidator<T>(ValidatorHandler<T>);

impl<T> std::fmt::Debug for ValValidator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValValidator").field(&"{...}").finish()
    }
}

impl<T: ErasedTy> ValValidator<T> {
    pub fn invoke(&self, val: &T) -> bool {
        (self.0)(val)
    }

    #[cfg(feature = "sync")]
    pub fn from_fn(func: impl Fn(&T) -> bool + Send + Sync + 'static) -> Self {
        Self(Box::new(move |val| func(val)))
    }

    #[cfg(not(feature = "sync"))]
    pub fn from_fn(func: impl Fn(&T) -> bool + 'static) -> Self {
        Self(Box::new(move |val| func(val)))
    }
}

impl<T: ErasedTy + PartialEq> ValValidator<T> {
    pub fn equal(val: T) -> Self {
        Self(Box::new(move |inner_val| inner_val == &val))
    }

    pub fn contains(vals: Vec<T>) -> Self {
        Self(Box::new(move |inner_val| vals.contains(inner_val)))
    }
}

impl<T: ErasedTy> ValValidator<T> {
    pub fn equal2<K>(val: K) -> Self
    where
        K: ErasedTy + PartialEq<T>,
    {
        Self(Box::new(move |inner_val| &val == inner_val))
    }

    pub fn contains2<K>(vals: Vec<K>) -> Self
    where
        K: ErasedTy + PartialEq<T>,
    {
        Self(Box::new(move |inner_val| {
            vals.iter().any(|v| PartialEq::eq(v, &inner_val))
        }))
    }
}

impl<T: ErasedTy + PartialOrd> ValValidator<T> {
    pub fn range_full(start: T, end: T) -> Self {
        Self(Box::new(move |inner_val| {
            inner_val >= &start && inner_val <= &end
        }))
    }

    pub fn range_from(start: T) -> Self {
        Self(Box::new(move |inner_val| inner_val >= &start))
    }

    pub fn range_to(end: T) -> Self {
        Self(Box::new(move |inner_val| inner_val <= &end))
    }
}
