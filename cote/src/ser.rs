use crate::ctx::RunningCtx;
use aopt::prelude::ErasedTy;
use aopt::prelude::ServicesValExt;
use aopt::Error;

pub trait CoteServiceExt {
    fn rctx(&self) -> Result<&RunningCtx, Error>;

    fn rctx_mut(&mut self) -> Result<&mut RunningCtx, Error>;

    fn set_rctx(&mut self, ctx: RunningCtx) -> &mut Self;

    fn take_rctx(&mut self) -> Result<RunningCtx, aopt::Error> {
        Ok(std::mem::take(self.rctx_mut()?))
    }

    fn sub_parsers<P: ErasedTy>(&self) -> Result<&Vec<P>, Error>;

    fn sub_parsers_mut<P: ErasedTy>(&mut self) -> Result<&mut Vec<P>, Error>;

    fn sub_parser<P: ErasedTy>(&self, id: usize) -> Result<Option<&P>, aopt::Error> {
        Ok(self.sub_parsers()?.get(id))
    }

    fn sub_parser_mut<P: ErasedTy>(&mut self, id: usize) -> Result<Option<&mut P>, aopt::Error> {
        Ok(self.sub_parsers_mut()?.get_mut(id))
    }

    fn add_parser<P: ErasedTy>(&mut self, parser: P) -> Result<&mut Self, Error> {
        self.sub_parsers_mut()?.push(parser);
        Ok(self)
    }

    fn rem_parser<P: ErasedTy>(&mut self, id: usize) -> Result<P, Error> {
        Ok(self.sub_parsers_mut()?.remove(id))
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

    fn sub_parsers<P: ErasedTy>(&self) -> Result<&Vec<P>, Error> {
        self.sve_val::<Vec<P>>()
    }

    fn sub_parsers_mut<P: ErasedTy>(&mut self) -> Result<&mut Vec<P>, Error> {
        self.sve_val_mut::<Vec<P>>()
    }
}
