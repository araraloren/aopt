use crate::map::ErasedTy;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub type ValidatorHandler<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;
    }
    else {
        pub type ValidatorHandler<T> = Box<dyn Fn(&T) -> bool>;
    }
}

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

    pub fn from(func: impl Fn(&T) -> bool + 'static) -> Self {
        Self(Box::new(move |val| func(val)))
    }
}

impl<T: ErasedTy + PartialEq> ValValidator<T> {
    pub fn equal(val: T) -> Self {
        Self(Box::new(move |inner_val| inner_val == &val))
    }

    pub fn equals(vals: Vec<T>) -> Self {
        Self(Box::new(move |inner_val| vals.contains(inner_val)))
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
