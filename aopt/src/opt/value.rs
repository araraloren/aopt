use crate::ctx::Ctx;
use crate::Error;
use crate::RawVal;
use std::any::Any;
use std::fmt::Debug;

pub trait RawValParser<Opt, Val> {
    fn parse(&mut self, opt: &Opt, val: Option<RawVal>, ctx: &Ctx) -> Result<Val, Error>;
}

impl<Opt, Val, Func> RawValParser<Opt, Val> for Func
where
    Func: FnMut(&Opt, Option<RawVal>, &Ctx) -> Result<Val, Error>,
{
    fn parse(&mut self, opt: &Opt, val: Option<RawVal>, ctx: &Ctx) -> Result<Val, Error> {
        (self)(opt, val, ctx)
    }
}

pub struct OptValParser<Opt, Val>(Box<dyn RawValParser<Opt, Val>>)
where
    Val: 'static,
    Opt: 'static;

impl<Opt, Val> OptValParser<Opt, Val>
where
    Opt: 'static,
    Val: 'static,
{
    pub fn new(parser: impl RawValParser<Opt, Val> + 'static) -> Self {
        Self(Box::new(parser))
    }

    pub fn invoke(&mut self, opt: &Opt, val: Option<RawVal>, ctx: &Ctx) -> Result<Val, Error> {
        self.0.parse(opt, val, ctx)
    }

    pub fn into_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
}

impl<Opt, Val> Debug for OptValParser<Opt, Val> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OptValParser").field(&"{...}").finish()
    }
}
