use std::ops::{Deref, DerefMut};

use crate::Error;
use crate::{ctx::Ctx, RawVal};

#[derive(Debug, Clone, Default)]
pub struct ReturnVal {
    error: Error,

    ctx: Ctx,
}

impl ReturnVal {
    pub fn new(ctx: Ctx) -> Self {
        Self {
            ctx,
            error: Error::Null,
        }
    }

    pub fn with_error(mut self, error: Error) -> Self {
        self.error = error;
        self
    }

    pub fn set_error(&mut self, error: Error) -> &mut Self {
        self.error = error;
        self
    }

    pub fn error(&self) -> &Error {
        &self.error
    }

    pub fn ctx(&self) -> &Ctx {
        &self.ctx
    }

    pub fn args(&self) -> &[RawVal] {
        self.ctx.args().as_slice()
    }

    pub fn status(&self) -> bool {
        self.error.is_null()
    }

    /// Check the error, return [`Ctx`] if error is null.
    pub fn ok_ctx(&self) -> Result<&Ctx, Error> {
        if !self.error.is_null() {
            Err(self.error.clone())
        } else {
            Ok(&self.ctx)
        }
    }

    pub fn take_ctx(&mut self) -> Ctx {
        std::mem::take(&mut self.ctx)
    }

    pub fn take_error(&mut self) -> Error {
        std::mem::take(&mut self.error)
    }

    pub fn clone_args(&self) -> Vec<RawVal> {
        let args = self.ctx.args().as_ref();

        args.clone().into_inner()
    }
}

impl Deref for ReturnVal {
    type Target = Ctx;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for ReturnVal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl From<ReturnVal> for bool {
    fn from(value: ReturnVal) -> Self {
        value.status()
    }
}

impl<'a> From<&'a ReturnVal> for bool {
    fn from(value: &'a ReturnVal) -> Self {
        value.status()
    }
}

impl<'a> From<&'a mut ReturnVal> for bool {
    fn from(value: &'a mut ReturnVal) -> Self {
        value.status()
    }
}
