use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
use crate::opt::RawValParser;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::ValService;
use crate::set::Set;
use crate::set::SetExt;
use crate::Error;
use crate::RawVal;
use crate::Uid;

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

pub struct CtxValue<T>(T);

impl<T: Debug> Debug for CtxValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CtxValue").field(&self.0).finish()
    }
}

impl<T> Deref for CtxValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for CtxValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set, T: RawValParser<<S as Set>::Opt>> ExtractCtx<S> for CtxValue<T> {
    type Error = Error;

    fn extract(uid: Uid, set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(CtxValue(
            T::parse(set.opt(uid)?, ctx.arg(), ctx).map_err(|e| e.into())?,
        ))
    }
}
