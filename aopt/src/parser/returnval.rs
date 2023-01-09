use std::ops::{Deref, DerefMut};

use crate::{ctx::Ctx, RawVal};

#[derive(Debug, Clone, Default)]
pub struct ReturnVal {
    status: bool,

    ctx: Ctx,

    args: Vec<RawVal>,
}

impl ReturnVal {
    pub fn new(ctx: Ctx, status: bool) -> Self {
        let args = ctx.args().into_inner();

        Self { status, ctx, args }
    }

    pub fn ctx(&self) -> &Ctx {
        &self.ctx
    }

    pub fn args(&self) -> &Vec<RawVal> {
        &self.args
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn take_ctx(&mut self) -> Ctx {
        std::mem::take(&mut self.ctx)
    }

    pub fn take_args(&mut self) -> Vec<RawVal> {
        std::mem::take(&mut self.args)
    }

    pub fn into_args(mut self) -> Vec<RawVal> {
        self.take_args()
    }
}

impl Deref for ReturnVal {
    type Target = Vec<RawVal>;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl DerefMut for ReturnVal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.args
    }
}
