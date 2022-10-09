use std::ops::Deref;
use std::ops::DerefMut;

use super::ExtractCtx;
use crate::arg::Args;
use crate::opt::OptStyle;
use crate::ser::Services;
use crate::set::Set;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Invoke context using for [`InvokeService`](crate::ser::InvokeService).
#[derive(Debug, Clone, Default)]
pub struct Ctx {
    uid: Uid,

    name: Str,

    pre: Option<Str>,

    style: OptStyle,

    dsb: bool,

    arg: Option<Str>,

    idx: usize,

    len: usize,

    args: Args,
}

impl Ctx {
    /// The uid of matching context.
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    /// The name of matching context.
    pub fn with_name(mut self, name: Str) -> Self {
        self.name = name;
        self
    }

    /// The prefix of matching context.
    pub fn with_pre(mut self, prefix: Option<Str>) -> Self {
        self.pre = prefix;
        self
    }

    /// The style of matching context.
    pub fn with_sty(mut self, style: OptStyle) -> Self {
        self.style = style;
        self
    }

    /// The deactivate value of matching context.
    pub fn with_dsb(mut self, disable: bool) -> Self {
        self.dsb = disable;
        self
    }

    /// The argument of matching context.
    pub fn with_arg(mut self, argument: Option<Str>) -> Self {
        self.arg = argument;
        self
    }

    /// The arguments of matching context.
    pub fn with_args(mut self, args: Args) -> Self {
        self.args = args;
        self
    }

    /// The index of matching context.
    pub fn with_idx(mut self, idx: usize) -> Self {
        self.idx = idx;
        self
    }

    /// The total of matching context.
    pub fn with_len(mut self, len: usize) -> Self {
        self.len = len;
        self
    }

    pub fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    pub fn set_name(&mut self, name: Str) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_pre(&mut self, prefix: Option<Str>) -> &mut Self {
        self.pre = prefix;
        self
    }

    pub fn set_sty(&mut self, style: OptStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_dsb(&mut self, disable: bool) -> &mut Self {
        self.dsb = disable;
        self
    }

    pub fn set_arg(&mut self, argument: Option<Str>) -> &mut Self {
        self.arg = argument;
        self
    }

    pub fn set_args(&mut self, args: Args) -> &mut Self {
        self.args = args;
        self
    }

    /// The index of matching context.
    pub fn set_idx(&mut self, index: usize) -> &mut Self {
        self.idx = index;
        self
    }

    /// The total of matching context.
    pub fn set_len(&mut self, total: usize) -> &mut Self {
        self.len = total;
        self
    }

    pub fn pre(&self) -> Option<&Str> {
        self.pre.as_ref()
    }

    pub fn sty(&self) -> OptStyle {
        self.style
    }

    pub fn name(&self) -> &Str {
        &self.name
    }

    pub fn args(&self) -> &Args {
        &self.args
    }

    pub fn uid(&self) -> Uid {
        self.uid
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Matching argument generate by [`guess_style`](crate::policy::Guess).
    pub fn arg(&self) -> Option<&Str> {
        self.arg.as_ref()
    }

    pub fn dsb(&self) -> bool {
        self.dsb
    }

    /// Get argument from [`Args`]
    pub fn orig_arg(&self) -> Option<&Str> {
        self.args.get(self.idx.saturating_sub(1))
    }
}

impl<S: Set> ExtractCtx<S> for Ctx {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ctx.clone())
    }
}

/// The uid of [`Match`](crate::proc::Match).
#[derive(Debug)]
pub struct CtxUid(Uid);

impl Deref for CtxUid {
    type Target = Uid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxUid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxUid {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxUid(ctx.uid()))
    }
}

/// The name of [`Match`](crate::proc::Match).
#[derive(Debug)]
pub struct CtxName(Str);

impl Deref for CtxName {
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxName {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxName(ctx.name().clone()))
    }
}

/// The prefix of [`Match`](crate::proc::Match).
#[derive(Debug)]
pub struct CtxPrefix(Option<Str>);

impl Deref for CtxPrefix {
    type Target = Option<Str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxPrefix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxPrefix {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxPrefix(ctx.pre().cloned()))
    }
}

/// The style of [`Match`](crate::proc::Match).
#[derive(Debug)]
pub struct CtxStyle(OptStyle);

impl Deref for CtxStyle {
    type Target = OptStyle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxStyle {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxStyle(ctx.sty()))
    }
}

/// The disable value of [`Match`](crate::proc::Match).
#[derive(Debug)]
pub struct CtxDisbale(bool);

impl Deref for CtxDisbale {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxDisbale {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxDisbale {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxDisbale(ctx.dsb()))
    }
}

/// The argument generated in [`Match`](crate::proc::Match).
#[derive(Debug)]
pub struct CtxMatArg(Option<Str>);

impl Deref for CtxMatArg {
    type Target = Option<Str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxMatArg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxMatArg {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxMatArg(ctx.arg().cloned()))
    }
}

/// The idx value set during parsing in [`Policy`](crate::policy::Policy).
#[derive(Debug)]
pub struct CtxIdx(usize);

impl Deref for CtxIdx {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxIdx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxIdx {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxIdx(ctx.idx()))
    }
}

/// The len value set during parsing in [`Policy`](crate::policy::Policy).
#[derive(Debug)]
pub struct CtxLen(usize);

impl Deref for CtxLen {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CtxLen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> ExtractCtx<S> for CtxLen {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxLen(ctx.idx()))
    }
}
