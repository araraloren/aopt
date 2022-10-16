use super::ExtractCtx;
use crate::arg::Args;
use crate::opt::OptStyle;
use crate::ser::Services;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::RawString;
use crate::Str;
use crate::Uid;

#[derive(Debug, Clone, Default)]
pub struct NOACtx {
    uid: Uid,

    idx: usize,

    len: usize,

    style: OptStyle,

    args: Arc<Args>,
}

impl NOACtx {
    pub fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    pub fn set_sty(&mut self, style: OptStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_args(&mut self, args: Arc<Args>) -> &mut Self {
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

    pub fn sty(&self) -> OptStyle {
        self.style
    }

    pub fn args(&self) -> &Arc<Args> {
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

    pub fn arg(&self) -> Option<&RawString> {
        self.args.get(self.idx().saturating_sub(1))
    }
}

#[derive(Debug, Clone)]
pub enum Ctx {
    NOA(NOACtx),
    OPT(OPTCtx),
    NULL,
}

impl Ctx {
    pub fn new_opt() -> Self {
        Self::OPT(OPTCtx::default())
    }

    pub fn new_noa() -> Self {
        Self::NOA(NOACtx::default())
    }

    pub fn is_opt(&self) -> bool {
        matches!(self, Self::OPT(_))
    }

    pub fn is_noa(&self) -> bool {
        matches!(self, Self::NOA(_))
    }

    pub fn opt(&self) -> Result<&OPTCtx, Error> {
        match self {
            Ctx::OPT(opt) => Ok(opt),
            _ => Err(Error::raise_error("OPTCtx excepted")),
        }
    }

    pub fn noa(&self) -> Result<&NOACtx, Error> {
        match self {
            Ctx::NOA(noa) => Ok(noa),
            _ => Err(Error::raise_error("NOACtx excepted")),
        }
    }

    pub fn opt_mut(&mut self) -> Result<&mut OPTCtx, Error> {
        match self {
            Ctx::OPT(opt) => Ok(opt),
            _ => Err(Error::raise_error("OPTCtx excepted")),
        }
    }

    pub fn noa_mut(&mut self) -> Result<&mut NOACtx, Error> {
        match self {
            Ctx::NOA(noa) => Ok(noa),
            _ => Err(Error::raise_error("NOACtx excepted")),
        }
    }
}

/// Invoke context using for [`InvokeService`](crate::ser::InvokeService).
#[derive(Debug, Clone, Default)]
pub struct OPTCtx {
    uid: Uid,

    name: Str,

    pre: Option<Str>,

    style: OptStyle,

    dsb: bool,

    arg: Option<Arc<RawString>>,

    idx: usize,

    len: usize,

    args: Arc<Args>,
}

impl OPTCtx {
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

    pub fn set_arg(&mut self, argument: Option<Arc<RawString>>) -> &mut Self {
        self.arg = argument;
        self
    }

    pub fn set_args(&mut self, args: Arc<Args>) -> &mut Self {
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
    pub fn arg(&self) -> Option<&Arc<RawString>> {
        self.arg.as_ref()
    }

    pub fn dsb(&self) -> bool {
        self.dsb
    }

    /// Get argument from [`Args`]
    pub fn orig_arg(&self) -> Option<&RawString> {
        self.args.get(self.idx.saturating_sub(1))
    }
}

impl<S: Set> ExtractCtx<S> for Ctx {
    type Error = Error;

    fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ctx.clone())
    }
}

// /// The uid of [`Match`](crate::proc::Match).
// #[derive(Debug)]
// pub struct CtxUid(Uid);

// impl Deref for CtxUid {
//     type Target = Uid;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxUid {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxUid {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxUid(ctx.uid()))
//     }
// }

// /// The name of [`Match`](crate::proc::Match).
// #[derive(Debug)]
// pub struct CtxName(Str);

// impl Deref for CtxName {
//     type Target = Str;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxName {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxName {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxName(ctx.name().clone()))
//     }
// }

// /// The prefix of [`Match`](crate::proc::Match).
// #[derive(Debug)]
// pub struct CtxPrefix(Option<Str>);

// impl Deref for CtxPrefix {
//     type Target = Option<Str>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxPrefix {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxPrefix {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxPrefix(ctx.pre().cloned()))
//     }
// }

// /// The style of [`Match`](crate::proc::Match).
// #[derive(Debug)]
// pub struct CtxStyle(OptStyle);

// impl Deref for CtxStyle {
//     type Target = OptStyle;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxStyle {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxStyle {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxStyle(ctx.sty()))
//     }
// }

// /// The disable value of [`Match`](crate::proc::Match).
// #[derive(Debug)]
// pub struct CtxDisbale(bool);

// impl Deref for CtxDisbale {
//     type Target = bool;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxDisbale {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxDisbale {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxDisbale(ctx.dsb()))
//     }
// }

// /// The argument generated in [`Match`](crate::proc::Match).
// #[derive(Debug)]
// pub struct CtxMatArg(Option<Arc<OsStr>>);

// impl Deref for CtxMatArg {
//     type Target = Option<Arc<OsStr>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxMatArg {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxMatArg {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxMatArg(ctx.arg().cloned()))
//     }
// }

// /// The idx value set during parsing in [`Policy`](crate::policy::Policy).
// #[derive(Debug)]
// pub struct CtxIdx(usize);

// impl Deref for CtxIdx {
//     type Target = usize;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxIdx {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxIdx {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxIdx(ctx.idx()))
//     }
// }

// /// The len value set during parsing in [`Policy`](crate::policy::Policy).
// #[derive(Debug)]
// pub struct CtxLen(usize);

// impl Deref for CtxLen {
//     type Target = usize;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for CtxLen {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl<S: Set> ExtractCtx<S> for CtxLen {
//     type Error = Error;

//     fn extract(_uid: Uid, _set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
//         Ok(CtxLen(ctx.idx()))
//     }
// }
