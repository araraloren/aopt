use crate::ctx::RunningCtx;
use aopt::prelude::ErasedTy;
use aopt::prelude::ServicesValExt;
use aopt::Error;
use std::collections::HashMap;

pub trait CoteServiceExt {
    fn rctx(&self) -> Result<&RunningCtx, Error>;

    fn rctx_mut(&mut self) -> Result<&mut RunningCtx, Error>;

    fn set_rctx(&mut self, ctx: RunningCtx) -> &mut Self;

    fn take_rctx(&mut self) -> Result<RunningCtx, aopt::Error> {
        Ok(std::mem::take(self.rctx_mut()?))
    }

    fn parsers<P: ErasedTy>(&self) -> Result<&HashMap<String, P>, Error>;

    fn parsers_mut<P: ErasedTy>(&mut self) -> Result<&mut HashMap<String, P>, Error>;

    fn parser_iter<P: ErasedTy>(
        &self,
    ) -> Result<std::collections::hash_map::Values<'_, String, P>, aopt::Error> {
        self.parsers().map(|parsers| parsers.values())
    }

    fn parser<P: ErasedTy>(&self, name: &str) -> Result<&P, aopt::Error> {
        let parsers = self.parsers()?;
        parsers
            .get(name)
            .ok_or_else(|| aopt::raise_error!("Can not find parser by name: {}", name))
    }

    fn parser_mut<P: ErasedTy>(&mut self, name: &str) -> Result<&mut P, aopt::Error> {
        let parsers = self.parsers_mut()?;
        parsers
            .get_mut(name)
            .ok_or_else(|| aopt::raise_error!("Can not find parser by name: {}", name))
    }

    fn add_parser<P: ErasedTy>(
        &mut self,
        name: impl Into<String>,
        parser: P,
    ) -> Result<&mut Self, Error> {
        self.parsers_mut()?.insert(name.into(), parser);
        Ok(self)
    }

    fn rem_parser<P: ErasedTy>(&mut self, name: &str) -> Result<Option<P>, Error> {
        Ok(self.parsers_mut()?.remove(name))
    }
}

impl<T: ServicesValExt> CoteServiceExt for T {
    fn rctx(&self) -> Result<&RunningCtx, aopt::Error> {
        self.sve_val()
    }

    fn rctx_mut(&mut self) -> Result<&mut RunningCtx, aopt::Error> {
        self.sve_val_mut()
    }

    fn set_rctx(&mut self, ctx: RunningCtx) -> &mut Self {
        self.sve_insert(ctx);
        self
    }

    fn parsers<P: ErasedTy>(&self) -> Result<&HashMap<String, P>, Error> {
        self.sve_val()
    }

    fn parsers_mut<P: ErasedTy>(&mut self) -> Result<&mut HashMap<String, P>, Error> {
        self.sve_val_mut()
    }
}
