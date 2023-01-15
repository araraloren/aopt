use std::fmt::Debug;

use crate::map::ErasedTy;
use crate::Error;

use super::AnyValue;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub trait InitializeValue<T: ErasedTy>: Send + Sync {
            type Error: Into<Error>;

            fn prepare_value(&mut self) -> Result<T, Self::Error>;
        }

        impl<Func, Err, T: ErasedTy> InitializeValue<T> for Func
        where
            Err: Into<Error>,
            Func: FnMut() -> Result<T, Err> + Send + Sync,
        {
            type Error = Err;

            fn prepare_value(&mut self) -> Result<T, Self::Error> {
                (self)()
            }
        }

        pub type InitHandler<T> = Box<dyn FnMut(&mut T) -> Result<(), Error> + Send + Sync>;
    }
    else {
        pub trait InitializeValue<T: ErasedTy> {
            type Error: Into<Error>;

            fn prepare_value(&mut self) -> Result<T, Self::Error>;
        }

        impl<Func, Err, T: ErasedTy> InitializeValue<T> for Func
        where
            Err: Into<Error>,
            Func: FnMut() -> Result<T, Err>,
        {
            type Error = Err;

            fn prepare_value(&mut self) -> Result<T, Self::Error> {
                (self)()
            }
        }

        pub type InitHandler<T> = Box<dyn FnMut(&mut T) -> Result<(), Error>>;
    }
}

pub struct ValInitializer(InitHandler<AnyValue>);

impl Debug for ValInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValInitializer").field(&"{...}").finish()
    }
}

impl ValInitializer {
    pub fn new<U: ErasedTy>(mut init: impl InitializeValue<Vec<U>> + 'static) -> Self {
        Self(Box::new(move |erased_val| {
            erased_val.set(init.prepare_value().map_err(Into::into)?);
            Ok(())
        }))
    }

    pub fn with<U: Copy + ErasedTy>(val: U) -> Self {
        Self(Box::new(move |erased_val| {
            erased_val.set(vec![val]);
            Ok(())
        }))
    }

    pub fn with_clone<U: Clone + ErasedTy>(val: U) -> Self {
        Self(Box::new(move |erased_val| {
            erased_val.set(vec![val.clone()]);
            Ok(())
        }))
    }

    pub fn with_vec<U: Clone + ErasedTy>(vals: Vec<U>) -> Self {
        Self(Box::new(move |erased_val| {
            erased_val.set(vals.clone());
            Ok(())
        }))
    }

    pub fn fallback() -> Self {
        Self(Box::new(|_| Ok(())))
    }

    pub fn invoke(&mut self, arg: &mut AnyValue) -> Result<(), Error> {
        (self.0)(arg)
    }
}
