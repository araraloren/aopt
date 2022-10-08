use std::any::Any;
use std::fmt::Debug;

use crate::ctx::Ctx;
use crate::prelude::Services;
use crate::{Error, Str};

type InnerCallbackType<Opt> =
    Box<dyn FnMut(&mut Opt, &mut Services, &Ctx) -> Result<Option<Str>, Error>>;

#[derive(Default)]
pub struct OptCallback<Opt>(Option<InnerCallbackType<Opt>>)
where
    Opt: 'static;

impl<Opt> OptCallback<Opt>
where
    Opt: 'static,
{
    pub fn new<H>(handler: H) -> Self
    where
        H: FnMut(&mut Opt, &mut Services, &Ctx) -> Result<Option<Str>, Error> + 'static,
    {
        Self(Some(Box::new(handler)))
    }

    pub fn invoke(
        &mut self,
        opt: &mut Opt,
        ser: &mut Services,
        ctx: &Ctx,
    ) -> Result<Option<Str>, Error> {
        if let Some(func) = &mut self.0 {
            (func)(opt, ser, ctx)
        } else {
            Ok(None)
        }
    }

    pub fn into_any(&mut self) -> Box<dyn Any> {
        let _self = std::mem::replace(self, Self(None));
        Box::new(_self)
    }
}

impl<Opt, H> From<H> for OptCallback<Opt>
where
    H: FnMut(&mut Opt, &mut Services, &Ctx) -> Result<Option<Str>, Error> + 'static,
{
    fn from(handler: H) -> Self {
        Self::new(handler)
    }
}

impl<Opt> Debug for OptCallback<Opt> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OptCallback").field(&"{...}").finish()
    }
}
