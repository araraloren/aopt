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

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        unsafe impl Send for ValValidator { }

        unsafe impl Sync for ValValidator { }
    }
}

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

macro_rules! num_validator {
    ($num:ty, $name:ident) => {
        pub fn $name() -> Self {
            fn _validator(
                _: &str,
                val: Option<&RawVal>,
                _: bool,
                _: (usize, usize),
            ) -> Result<bool, Error> {
                Ok(val
                    .and_then(|v| v.get_str())
                    .and_then(|v| v.parse::<$num>().ok())
                    .is_some())
            }

            Self::new(_validator)
        }
    };
}

impl ValValidator {
    num_validator!(i8, i8_validator);

    num_validator!(i16, i16_validator);

    num_validator!(i32, i32_validator);

    num_validator!(i64, i64_validator);

    num_validator!(u8, u8_validator);

    num_validator!(u16, u16_validator);

    num_validator!(u32, u32_validator);

    num_validator!(u64, u64_validator);

    num_validator!(f32, f32_validator);

    num_validator!(f64, f64_validator);

    num_validator!(usize, usize_validator);

    num_validator!(isize, isize_validator);

    pub fn bool_validator(deactivate_style: bool) -> Self {
        Self::new(
            move |_: &str,
                  val: Option<&RawVal>,
                  disable: bool,
                  _: (usize, usize)|
                  -> Result<bool, Error> {
                if let Some(val) = val.and_then(|v| v.get_str()) {
                    if deactivate_style && disable && val == crate::opt::BOOL_FALSE
                        || !deactivate_style && !disable && val == crate::opt::BOOL_TRUE
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
        )
    }

    pub fn str_validator() -> Self {
        Self::new(
            move |_: &str,
                  val: Option<&RawVal>,
                  _: bool,
                  _: (usize, usize)|
                  -> Result<bool, Error> {
                Ok(val.map(|v| v.get_str().is_some()).unwrap_or_default())
            },
        )
    }

    pub fn some_validator() -> Self {
        Self::new(
            move |_: &str,
                  val: Option<&RawVal>,
                  _: bool,
                  _: (usize, usize)|
                  -> Result<bool, Error> { Ok(val.is_some()) },
        )
    }

    pub fn null_validator() -> Self {
        Self::new(
            |_: &str, _: Option<&RawVal>, _: bool, _: (usize, usize)| -> Result<bool, Error> {
                Ok(true)
            },
        )
    }
}

pub trait ValValidatorExt {
    type Valid;

    fn val_validator() -> Self::Valid;
}

macro_rules! impl_validator_ext_for {
    ($num:ty, $name:ident) => {
        impl ValValidatorExt for $num {
            type Valid = ValValidator;

            fn val_validator() -> Self::Valid {
                ValValidator::$name()
            }
        }
    };
}

impl_validator_ext_for!(i8, i8_validator);

impl_validator_ext_for!(i16, i16_validator);

impl_validator_ext_for!(i32, i32_validator);

impl_validator_ext_for!(i64, i64_validator);

impl_validator_ext_for!(u8, u8_validator);

impl_validator_ext_for!(u16, u16_validator);

impl_validator_ext_for!(u32, u32_validator);

impl_validator_ext_for!(u64, u64_validator);

impl_validator_ext_for!(f32, f32_validator);

impl_validator_ext_for!(f64, f64_validator);

impl_validator_ext_for!(str, str_validator);

impl_validator_ext_for!(usize, usize_validator);

impl_validator_ext_for!(isize, isize_validator);

impl ValValidatorExt for () {
    type Valid = ValValidator;

    fn val_validator() -> Self::Valid {
        ValValidator::null_validator()
    }
}

pub trait ValValidatorExt2 {
    type Valid;

    fn val_validator(deactivate_style: bool) -> Self::Valid;
}

impl ValValidatorExt2 for bool {
    type Valid = ValValidator;

    fn val_validator(deactivate_style: bool) -> Self::Valid {
        ValValidator::bool_validator(deactivate_style)
    }
}
