use std::borrow::Cow;
use std::ffi::OsStr;
use std::ops::{Deref, DerefMut};

use crate::ctx::Ctx;
use crate::Error;

/// Return value for [`Policy`](crate::parser::Policy).
#[derive(Debug, Clone, Default)]
pub struct ReturnVal<'a> {
    ctx: Ctx<'a>,

    failure: Option<Error>,
}

impl<'a> ReturnVal<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
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

    pub fn ctx(&self) -> &Ctx<'a> {
        &self.ctx
    }

    pub fn args(&self) -> &[Cow<'a, OsStr>] {
        self.ctx.args().as_slice()
    }

    /// The [`status`](ReturnVal::status) is true if parsing successes
    /// otherwise it will be false if any [`failure`](Error::is_failure) raised.
    pub fn status(&self) -> bool {
        self.failure.is_none()
    }

    /// Unwrap the [`Ctx`] from [`ReturnVal`].
    pub fn unwrap(self) -> Ctx<'a> {
        Result::unwrap(if self.failure.is_none() {
            Ok(self.ctx)
        } else {
            Err(self.failure)
        })
    }

    pub fn ok(self) -> Result<Ctx<'a>, Error> {
        if let Some(failure) = self.failure {
            Err(failure)
        } else {
            Ok(self.ctx)
        }
    }

    pub fn take_ctx(&mut self) -> Ctx<'a> {
        std::mem::take(&mut self.ctx)
    }

    pub fn take_failure(&mut self) -> Option<Error> {
        self.failure.take()
    }

    pub fn clone_args(&self) -> Vec<Cow<'a, OsStr>> {
        self.ctx.args().clone().unwrap_or_clone()
    }
}

impl<'a> Deref for ReturnVal<'a> {
    type Target = Ctx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'a> DerefMut for ReturnVal<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl From<ReturnVal<'_>> for bool {
    fn from(value: ReturnVal<'_>) -> Self {
        value.status()
    }
}

impl<'a> From<&'a ReturnVal<'_>> for bool {
    fn from(value: &'a ReturnVal<'_>) -> Self {
        value.status()
    }
}

impl<'a> From<&'a mut ReturnVal<'_>> for bool {
    fn from(value: &'a mut ReturnVal<'_>) -> Self {
        value.status()
    }
}
