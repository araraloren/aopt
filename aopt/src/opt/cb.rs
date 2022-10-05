use std::any::Any;
use std::fmt::Debug;

use crate::ctx::Context;
use crate::prelude::Services;
use crate::{Error, Str};

type InnerCallbackType<T> =
    Box<dyn FnMut(&mut T, &Context, &mut Services) -> Result<Option<Str>, Error>>;

#[derive(Default)]
pub struct OptCallback<T>(Option<InnerCallbackType<T>>)
where
    T: 'static;

impl<T> OptCallback<T>
where
    T: 'static,
{
    pub fn new<H>(handler: H) -> Self
    where
        H: FnMut(&mut T, &Context, &mut Services) -> Result<Option<Str>, Error> + 'static,
    {
        Self(Some(Box::new(handler)))
    }

    pub fn invoke(
        &mut self,
        opt: &mut T,
        ctx: &Context,
        ser: &mut Services,
    ) -> Result<Option<Str>, Error> {
        if let Some(func) = &mut self.0 {
            (func)(opt, ctx, ser)
        } else {
            Ok(None)
        }
    }

    pub fn into_any(&mut self) -> Box<dyn Any> {
        let _self = std::mem::replace(self, Self(None));
        Box::new(_self)
    }
}

impl<T, H> From<H> for OptCallback<T>
where
    H: FnMut(&mut T, &Context, &mut Services) -> Result<Option<Str>, Error> + 'static,
{
    fn from(handler: H) -> Self {
        Self::new(handler)
    }
}

impl<T> Debug for OptCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ACallback")
            .field(&"private callback".to_string())
            .finish()
    }
}
