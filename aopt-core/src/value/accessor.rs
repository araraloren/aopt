use std::any::type_name;
use std::ffi::{OsStr, OsString};
use std::ops::{Deref, DerefMut};

use super::AnyValue;
use super::ErasedValue;
use super::RawValParser;
use super::ValInitializer;
use super::ValStorer;
use super::ValValidator;

use crate::ctx::Ctx;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::raise_error;
use crate::Error;

/// [`ValAccessor`] manage the option value and raw value.
///
/// # Example
/// ```rust
/// # use aopt_core::ctx::*;
/// # use aopt_core::opt::Action;
/// # use aopt_core::value::*;
/// # use aopt_core::Error;
/// #
/// # use std::ffi::OsStr;
/// #
/// # fn main() -> Result<(), Error> {
/// let ctx = Ctx::default().with_inner_ctx(InnerCtx::default());
/// {
///     let mut value = ValAccessor::fallback::<i32>();
///     let raw_value = OsStr::new("123");
///
///     value.initialize()?;
///     value.set(vec![1, 4]);
///     value.store_all(Some(&raw_value), &ctx, &Action::App)?;
///     assert_eq!(value.pop::<i32>(), Some(123));
///     assert_eq!(value.rawval()?, &raw_value);
/// }
/// {
///     let mut value =
///         ValAccessor::new(ValStorer::fallback::<i32>(), ValInitializer::new_values(vec![7]));
///     let raw_value = OsStr::new("42");
///
///     value.initialize()?;
///     value.store_all(Some(&raw_value), &ctx, &Action::Set)?;
///     assert_eq!(value.pop::<i32>(), Some(42));
///     assert_eq!(value.rawval()?, &raw_value);
/// }
/// {
///     let validator = ValValidator::range_from(-32i32);
///     let mut value = ValAccessor::new_validator(validator, ValInitializer::fallback());
///     let raw_value1 = OsStr::new("8");
///
///     value.initialize()?;
///     value.set(vec![1, 4]);
///     assert_eq!(
///         value.store_all(Some(&raw_value1), &ctx, &Action::App)?,
///         true
///     );
///     assert_eq!(value.pop::<i32>(), Some(8));
///     assert_eq!(value.rawval()?, &raw_value1);
///
///     let raw_value2 = OsStr::new("-66");
///
///     assert!(value.store_all(Some(&raw_value2), &ctx, &Action::App).is_err());
///     assert_eq!(value.pop::<i32>(), Some(4));
///     assert_eq!(value.rawval()?, &raw_value1);
/// }
/// {
///     let validator = ValValidator::range_to(-42);
///     let mut value =
///         ValAccessor::new_validator(validator, ValInitializer::new_values(vec![-88, 1]));
///     let raw_value1 = OsStr::new("-68");
///
///     value.initialize()?;
///     assert_eq!(
///         value.store_all(Some(&raw_value1), &ctx, &Action::Set)?,
///         true
///     );
///     assert_eq!(value.pop::<i32>(), Some(-68));
///     assert_eq!(value.rawval()?, &raw_value1);
///
///     let raw_value2 = OsStr::new("-20");
///
///     assert!(value.store_all(Some(&raw_value2), &ctx, &Action::App).is_err());
///     assert_eq!(value.pop::<i32>(), None);
///     assert_eq!(value.rawval()?, &raw_value1);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ValAccessor {
    any_value: AnyValue,

    rawval: Vec<OsString>,

    storer: ValStorer,

    initializer: ValInitializer,
}

impl Default for ValAccessor {
    fn default() -> Self {
        Self::fallback::<String>()
    }
}

impl ValAccessor {
    pub fn new(storer: ValStorer, initializer: ValInitializer) -> Self {
        Self {
            any_value: AnyValue::default(),
            rawval: vec![],
            storer,
            initializer,
        }
    }

    pub fn new_validator<U: ErasedTy + RawValParser>(
        validator: ValValidator<U>,
        initializer: ValInitializer,
    ) -> Self {
        Self {
            any_value: AnyValue::default(),
            rawval: vec![],
            storer: ValStorer::new_validator(validator),
            initializer,
        }
    }

    pub fn fallback<U: ErasedTy + RawValParser>() -> Self {
        Self {
            any_value: AnyValue::default(),
            rawval: vec![],
            storer: ValStorer::fallback::<U>(),
            initializer: ValInitializer::fallback(),
        }
    }

    pub fn with_storer(mut self, storer: ValStorer) -> Self {
        self.storer = storer;
        self
    }

    pub fn with_initializer(mut self, initializer: ValInitializer) -> Self {
        self.initializer = initializer;
        self
    }

    pub fn set_storer(&mut self, storer: ValStorer) -> &mut Self {
        self.storer = storer;
        self
    }

    pub fn set_initializer(&mut self, initializer: ValInitializer) -> &mut Self {
        self.initializer = initializer;
        self
    }

    pub fn storer(&self) -> &ValStorer {
        &self.storer
    }

    pub fn initializer(&self) -> &ValInitializer {
        &self.initializer
    }

    pub fn storer_mut(&mut self) -> &mut ValStorer {
        &mut self.storer
    }

    pub fn initializer_mut(&mut self) -> &mut ValInitializer {
        &mut self.initializer
    }

    pub fn handlers(&mut self) -> (&mut Vec<OsString>, &mut AnyValue) {
        (&mut self.rawval, &mut self.any_value)
    }

    /// Parsing the raw value into typed value, save the raw value and result.
    /// The function will map the failure error to `Ok(false)`.
    pub fn store_all(
        &mut self,
        arg: Option<&OsStr>,
        ctx: &Ctx,
        act: &Action,
    ) -> Result<bool, Error> {
        match self.store(arg, ctx, act) {
            Ok(_) => {
                if let Some(raw) = arg {
                    self.rawval.push(raw.to_os_string());
                }
                Ok(true)
            }
            Err(e) => Err(e),
        }
    }
}

impl Deref for ValAccessor {
    type Target = AnyValue;

    fn deref(&self) -> &Self::Target {
        &self.any_value
    }
}

impl DerefMut for ValAccessor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.any_value
    }
}

impl ErasedValue for ValAccessor {
    fn initialize(&mut self) -> Result<(), Error> {
        let handler = &mut self.any_value;

        self.initializer.invoke(handler)
    }

    fn store(&mut self, arg: Option<&OsStr>, ctx: &Ctx, act: &Action) -> Result<(), Error> {
        let handler = &mut self.any_value;

        self.storer.invoke(arg, ctx, act, handler)
    }

    fn store_act<U: ErasedTy>(&mut self, val: U, _: &Ctx, act: &Action) -> Result<(), Error> {
        let handler = &mut self.any_value;
        let value = val;

        match act {
            Action::Set => {
                handler.set(vec![value]);
            }
            Action::App => {
                handler.push(value);
            }
            Action::Pop => {
                handler.pop::<U>();
            }
            Action::Cnt => {
                handler.entry::<u64>().or_insert(vec![0])[0] += 1;
            }
            Action::Clr => {
                handler.remove::<U>();
            }
            Action::Null => {
                // NOTHING
            }
        }
        Ok(())
    }

    fn val<U: ErasedTy>(&self) -> Result<&U, Error> {
        self.any_value.val()
    }

    fn val_mut<U: ErasedTy>(&mut self) -> Result<&mut U, Error> {
        self.any_value.val_mut()
    }

    fn vals<U: ErasedTy>(&self) -> Result<&Vec<U>, Error> {
        self.any_value.vals()
    }

    fn vals_mut<U: ErasedTy>(&mut self) -> Result<&mut Vec<U>, Error> {
        self.any_value.vals_mut()
    }

    fn take_val<U: ErasedTy>(&mut self) -> Result<U, Error> {
        self.any_value.pop().ok_or_else(|| {
            raise_error!(
                "can not take more value for type `{:?}` in ErasedVal(take_val)",
                type_name::<U>()
            )
        })
    }

    fn take_vals<U: ErasedTy>(&mut self) -> Result<Vec<U>, Error> {
        self.any_value.remove().ok_or_else(|| {
            raise_error!(
                "can not take more values for type `{:?}` in ErasedVal(take_vals)",
                type_name::<U>()
            )
        })
    }

    fn rawval(&self) -> Result<&OsString, Error> {
        self.rawval
            .last()
            .ok_or_else(|| raise_error!("no more raw value in accessor"))
    }

    fn rawval_mut(&mut self) -> Result<&mut OsString, Error> {
        self.rawval
            .last_mut()
            .ok_or_else(|| raise_error!("no more raw value in accessor"))
    }

    fn rawvals(&self) -> Result<&Vec<OsString>, Error> {
        Ok(&self.rawval)
    }

    fn rawvals_mut(&mut self) -> Result<&mut Vec<OsString>, Error> {
        Ok(&mut self.rawval)
    }

    fn take_rawval<U: ErasedTy>(&mut self) -> Result<OsString, Error> {
        self.rawval
            .pop()
            .ok_or_else(|| raise_error!("no more raw value in accessor"))
    }

    fn take_rawvals<U: ErasedTy>(&mut self) -> Result<Vec<OsString>, Error> {
        Ok(std::mem::take(&mut self.rawval))
    }
}
