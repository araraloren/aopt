use std::ffi::OsString;

use crate::args::Args;
use crate::ctx::Ctx;
use crate::opt::Style;
use crate::{Error, Uid};

#[derive(Debug, Clone, Default)]
pub struct Guess {
    pub uid: Uid,

    pub name: Option<String>,

    pub style: Style,

    pub arg: Option<OsString>,

    pub index: usize,

    pub total: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub orig: Args,

    pub args: Vec<OsString>,

    pub guess: Option<Guess>,
}

/// Return value for [`Policy`](crate::parser::Policy).
#[derive(Debug, Clone, Default)]
pub struct Return {
    ctx: Context,

    failure: Option<Error>,
}

impl Return {
    pub fn new(ctx: Ctx<'_>) -> Self {
        let args = ctx.args.into_iter().map(|v| v.to_os_string()).collect();

        Self {
            ctx: Context {
                orig: ctx.orig,
                args,
                guess: ctx.inner_ctx.map(|v| Guess {
                    uid: v.uid(),
                    name: v.name().map(|v| v.to_string()),
                    style: v.style(),
                    arg: v.arg().map(|v| v.to_os_string()),
                    index: v.idx(),
                    total: v.total(),
                }),
            },
            failure: None,
        }
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

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    pub fn args(&self) -> &[OsString] {
        &self.ctx.args
    }

    /// The original arguments passed by user.
    pub fn orig_args(&self) -> &Args {
        &self.ctx.orig
    }

    /// The [`status`](Return::status) is true if parsing successes
    /// otherwise it will be false if any [`failure`](Error::is_failure) raised.
    pub fn status(&self) -> bool {
        self.failure.is_none()
    }

    /// Unwrap the [`Ctx`] from [`Return`].
    pub fn unwrap(self) -> Context {
        Result::unwrap(if self.failure.is_none() {
            Ok(self.ctx)
        } else {
            Err(self.failure)
        })
    }

    pub fn ok(self) -> Result<Context, Error> {
        if let Some(failure) = self.failure {
            Err(failure)
        } else {
            Ok(self.ctx)
        }
    }

    pub fn take_ctx(&mut self) -> Context {
        std::mem::take(&mut self.ctx)
    }

    pub fn take_failure(&mut self) -> Option<Error> {
        self.failure.take()
    }

    pub fn take_args(&mut self) -> Vec<OsString> {
        std::mem::take(&mut self.ctx.args)
    }

    pub fn clone_args(&self) -> Vec<OsString> {
        self.ctx.args.clone()
    }
}

impl From<Return> for bool {
    fn from(value: Return) -> Self {
        value.status()
    }
}

impl From<&Return> for bool {
    fn from(value: &Return) -> Self {
        value.status()
    }
}

impl From<&mut Return> for bool {
    fn from(value: &mut Return) -> Self {
        value.status()
    }
}
