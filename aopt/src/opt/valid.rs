use crate::Error;
use crate::RawVal;
use std::any::Any;
use std::fmt::Debug;

pub trait RawValValidator {
    fn check(
        &mut self,
        name: &str,
        value: Option<&RawVal>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error>;
}

impl<Func> RawValValidator for Func
where
    Func: FnMut(&str, Option<&RawVal>, bool, (usize, usize)) -> Result<bool, Error>,
{
    fn check(
        &mut self,
        name: &str,
        value: Option<&RawVal>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error> {
        (self)(name, value, disable, index)
    }
}

pub struct ValValidator(Box<dyn RawValValidator>);

impl Default for ValValidator {
    fn default() -> Self {
        fn __default(
            _: &str,
            _: Option<&RawVal>,
            _: bool,
            _: (usize, usize),
        ) -> Result<bool, Error> {
            Ok(true)
        }

        Self::new(__default)
    }
}

impl ValValidator {
    pub fn new(inner: impl RawValValidator + 'static) -> Self {
        Self(Box::new(inner))
    }

    pub fn check(
        &mut self,
        name: &str,
        value: Option<&RawVal>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error> {
        self.0.check(name, value, disable, index)
    }

    pub fn into_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
}

impl Debug for ValValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValValidator").field(&"{...}").finish()
    }
}

impl ValValidator {
    pub fn i64_validator() -> Self {
        fn _validator(
            _: &str,
            val: Option<&RawVal>,
            _: bool,
            _: (usize, usize),
        ) -> Result<bool, Error> {
            Ok(val
                .and_then(|v| v.to_str())
                .and_then(|v| v.parse::<i64>().ok())
                .is_some())
        }

        Self::new(_validator)
    }
}
