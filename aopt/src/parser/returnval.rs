use std::ops::{Deref, DerefMut};

use crate::{ctx::Ctx, RawVal};

#[derive(Debug, Clone, Default)]
pub struct ReturnVal {
    status: bool,

    ctx: Ctx,
}

impl ReturnVal {
    pub fn new(ctx: Ctx, status: bool) -> Self {
        Self { status, ctx }
    }

    pub fn ctx(&self) -> &Ctx {
        &self.ctx
    }

    pub fn args(&self) -> &[RawVal] {
        self.ctx.args().as_slice()
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn take_ctx(&mut self) -> Ctx {
        std::mem::take(&mut self.ctx)
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

impl AsRef<bool> for ReturnVal {
    fn as_ref(&self) -> &bool {
        &self.status
    }
}
