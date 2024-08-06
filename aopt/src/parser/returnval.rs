use std::ops::{Deref, DerefMut};

use crate::Error;
use crate::{ctx::Ctx, RawVal};

/// Return value for [`Policy`](crate::parser::Policy).
#[derive(Debug, Clone, Default)]
pub struct ReturnVal {
    failure: Option<Error>,

    ctx: Ctx,
}

impl ReturnVal {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx, failure: None }
    }

    pub fn with_failure(mut self, failure: Error) -> Self {
        self.failure = Some(failure);
        self
    }

    pub fn set_failure(&mut self, failure: Error) -> &mut Self {
        self.failure = Some(failure);
        self
    }

    pub fn failure(&self) -> Option<&Error> {
        self.failure.as_ref()
    }

    pub fn ctx(&self) -> &Ctx {
        &self.ctx
    }

    pub fn args(&self) -> &[RawVal] {
        self.ctx.args().as_slice()
    }

    /// The [`status`](ReturnVal::status) is true if parsing successes
    /// otherwise it will be false if any [`failure`](Error::is_failure) raised.
    pub fn status(&self) -> bool {
        self.failure.is_some()
    }

    /// Unwrap the [`Ctx`] from [`ReturnVal`].
    pub fn unwrap(self) -> Ctx {
        Result::unwrap(if self.failure.is_some() {
            Ok(self.ctx)
        } else {
            Err(self.failure)
        })
    }

    pub fn ok(self) -> Result<Ctx, Error> {
        if let Some(failure) = self.failure {
            Err(failure)
        } else {
            Ok(self.ctx)
        }
    }

    pub fn take_ctx(&mut self) -> Ctx {
        std::mem::take(&mut self.ctx)
    }

    pub fn take_failure(&mut self) -> Option<Error> {
        self.failure.take()
    }

    pub fn clone_args(&self) -> Vec<RawVal> {
        let args = self.ctx.args().as_ref();

        args.clone().into()
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
