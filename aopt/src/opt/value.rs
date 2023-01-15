use crate::map::ErasedTy;
use crate::value::ErasedValHandler;
use crate::Error;
use crate::RawVal;

use super::Opt;

pub trait OptValueExt {
    fn val<T: ErasedTy>(&self) -> Result<&T, Error>;

    fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error>;

    fn vals<T: ErasedTy>(&self) -> Result<&Vec<T>, Error>;

    fn vals_mut<T: ErasedTy>(&mut self) -> Result<&mut Vec<T>, Error>;

    fn rawval(&self) -> Result<&RawVal, Error>;

    fn rawval_mut(&mut self) -> Result<&mut RawVal, Error>;

    fn rawvals(&self) -> Result<&Vec<RawVal>, Error>;

    fn rawvals_mut(&mut self) -> Result<&mut Vec<RawVal>, Error>;
}

impl<O: Opt> OptValueExt for O {
    fn val<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.accessor().val()
    }

    fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.accessor_mut().val_mut()
    }

    fn vals<T: ErasedTy>(&self) -> Result<&Vec<T>, Error> {
        self.accessor().vals()
    }

    fn vals_mut<T: ErasedTy>(&mut self) -> Result<&mut Vec<T>, Error> {
        self.accessor_mut().vals_mut()
    }

    fn rawval(&self) -> Result<&RawVal, Error> {
        self.accessor().rawval()
    }

    fn rawval_mut(&mut self) -> Result<&mut RawVal, Error> {
        self.accessor_mut().rawval_mut()
    }

    fn rawvals(&self) -> Result<&Vec<RawVal>, Error> {
        self.accessor().rawvals()
    }

    fn rawvals_mut(&mut self) -> Result<&mut Vec<RawVal>, Error> {
        self.accessor_mut().rawvals_mut()
    }
}
