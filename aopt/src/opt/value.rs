use crate::ctx::Ctx;
use crate::Arc;
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

pub struct ValParser<Opt, Val>(Box<dyn RawValParser<Opt, Val>>)
where
    Val: 'static,
    Opt: 'static;

impl<Opt, Val> ValParser<Opt, Val>
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

impl<Opt, Val> Debug for ValParser<Opt, Val> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OptValParser").field(&"{...}").finish()
    }
}

pub trait RawValValidator {
    fn check(
        &mut self,
        val: Option<Arc<RawVal>>,
        dsb: bool,
        idx: (usize, usize),
    ) -> Result<bool, Error>;
}

impl<Func> RawValValidator for Func
where
    Func: FnMut(Option<Arc<RawVal>>, bool, (usize, usize)) -> Result<bool, Error>,
{
    fn check(
        &mut self,
        val: Option<Arc<RawVal>>,
        dsb: bool,
        idx: (usize, usize),
    ) -> Result<bool, Error> {
        (self)(val, dsb, idx)
    }
}

pub struct ValValidator(Box<dyn RawValValidator>);

impl ValValidator {
    pub fn new(inner: impl RawValValidator + 'static) -> Self {
        Self(Box::new(inner))
    }

    pub fn check(
        &mut self,
        value: Option<Arc<RawVal>>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error> {
        self.0.check(value, disable, index)
    }

    pub fn into_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
}

impl Debug for ValValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValValidator").field(&"{...}").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValPolicy {
    Set,

    App,

    Pop,

    Cnt,

    Bool,

    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValType {
    Bool,

    Int,

    Uint,

    Flt,

    Str,

    Null,
}

pub struct Value<T: ?Sized>(pub T);

impl<Opt> RawValParser<Opt, i32> for Value<i32> where Opt: crate::opt::Opt {
    fn parse(&mut self, opt: &Opt, val: Option<RawVal>, ctx: &Ctx) -> Result<i32, Error> {
        assert_eq!(opt.val_ty(), ValType::Int);
        Ok(42)
    }
}