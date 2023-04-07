use std::fmt::Debug;

use crate::ctx::Ctx;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::trace_log;
use crate::Error;
use crate::RawVal;

use super::AnyValue;
use super::RawValParser;
use super::ValValidator;

#[cfg(feature = "sync")]
pub type StoreHandler<T> =
    Box<dyn FnMut(Option<&RawVal>, &Ctx, &Action, &mut T) -> Result<(), Error> + Send + Sync>;

#[cfg(not(feature = "sync"))]
pub type StoreHandler<T> =
    Box<dyn FnMut(Option<&RawVal>, &Ctx, &Action, &mut T) -> Result<(), Error>>;

/// [`ValStorer`] perform the value storing action.
pub struct ValStorer(StoreHandler<AnyValue>);

impl Debug for ValStorer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WriterHandler").field(&"{...}").finish()
    }
}

impl ValStorer {
    pub fn new(handler: StoreHandler<AnyValue>) -> Self {
        Self(handler)
    }

    pub fn fallback<U: ErasedTy + RawValParser>() -> Self {
        Self(Self::fallback_handler::<U>())
    }

    /// Create a [`ValStorer`] with a value validator.
    /// The [`invoke`](ValStorer::invoke) will return a [`failure`](Error::is_failure)
    /// if value check failed.
    pub fn new_validator<U: ErasedTy + RawValParser>(validator: ValValidator<U>) -> Self {
        Self(Self::validator(validator))
    }

    /// Invoke the inner value store handler on [`AnyValue`].
    pub fn invoke(
        &mut self,
        raw: Option<&RawVal>,
        ctx: &Ctx,
        act: &Action,
        arg: &mut AnyValue,
    ) -> Result<(), Error> {
        crate::trace_log!("Saving raw value({:?}) for {}", raw, ctx.uid()?);
        (self.0)(raw, ctx, act, arg)
    }

    pub fn validator<U: ErasedTy + RawValParser>(
        validator: ValValidator<U>,
    ) -> StoreHandler<AnyValue> {
        Box::new(
            move |raw: Option<&RawVal>, ctx: &Ctx, act: &Action, handler: &mut AnyValue| {
                let val = U::parse(raw, ctx).map_err(Into::into)?;

                if !validator.invoke(&val) {
                    trace_log!(
                        "Validator value storer failed, parsing {:?} -> {:?}",
                        raw,
                        val
                    );
                    Err(Error::raise_failure(format!(
                        "Value check failed for option Ctx {{ `{:?}` }}",
                        ctx,
                    )))
                } else {
                    trace_log!(
                        "Validator value storer okay, parsing {:?} -> {:?}",
                        raw,
                        val
                    );
                    act.store1(Some(val), handler);
                    Ok(())
                }
            },
        )
    }

    pub fn fallback_handler<U: ErasedTy + RawValParser>() -> StoreHandler<AnyValue> {
        Box::new(
            |raw: Option<&RawVal>, ctx: &Ctx, act: &Action, handler: &mut AnyValue| {
                let val = U::parse(raw, ctx).map_err(Into::into)?;

                trace_log!("Fallback value storer, parsing {:?} -> {:?}", raw, val);
                act.store1(Some(val), handler);
                Ok(())
            },
        )
    }
}

impl<U: ErasedTy + RawValParser> From<ValValidator<U>> for ValStorer {
    fn from(validator: ValValidator<U>) -> Self {
        Self::new_validator(validator)
    }
}

impl<U: ErasedTy + RawValParser> From<Option<ValValidator<U>>> for ValStorer {
    fn from(validator: Option<ValValidator<U>>) -> Self {
        if let Some(validator) = validator {
            Self::new_validator(validator)
        } else {
            Self::fallback::<U>()
        }
    }
}
