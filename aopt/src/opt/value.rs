use crate::map::Entry;
use crate::map::ErasedTy;
use crate::raise_error;
use crate::value::ErasedValue;
use crate::Error;
use crate::RawVal;

use super::Opt;

pub trait OptValueExt {
    fn val<T: ErasedTy>(&self) -> Result<&T, Error>;

    fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error>;

    fn vals<T: ErasedTy>(&self) -> Result<&Vec<T>, Error>;

    fn vals_mut<T: ErasedTy>(&mut self) -> Result<&mut Vec<T>, Error>;

    fn entry<T: ErasedTy>(&mut self) -> Entry<'_, Vec<T>>;

    fn rawval(&self) -> Result<&RawVal, Error>;

    fn rawval_mut(&mut self) -> Result<&mut RawVal, Error>;

    fn rawvals(&self) -> Result<&Vec<RawVal>, Error>;

    fn rawvals_mut(&mut self) -> Result<&mut Vec<RawVal>, Error>;

    fn filter<T: ErasedTy>(&mut self, f: impl FnMut(&T) -> bool) -> Result<Vec<T>, Error>;
}

impl<O: Opt> OptValueExt for O {
    fn val<T: ErasedTy>(&self) -> Result<&T, Error> {
        let uid = self.uid();

        self.accessor().val().map_err(|e| {
            e.cause(raise_error!(
                "Can not find value(ref) of `{}`({})",
                self.hint(),
                self.action(),
            ))
            .with_uid(uid)
        })
    }

    fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        let hint = self.hint().clone();
        let action = *self.action();
        let uid = self.uid();

        self.accessor_mut().val_mut().map_err(|e| {
            e.cause(raise_error!(
                "Can not find value(mut) of `{}`({})",
                hint,
                action
            ))
            .with_uid(uid)
        })
    }

    fn vals<T: ErasedTy>(&self) -> Result<&Vec<T>, Error> {
        let uid = self.uid();

        self.accessor().vals().map_err(|e| {
            e.cause(raise_error!(
                "Can not find values(ref) of `{}`({})",
                self.hint(),
                self.action(),
            ))
            .with_uid(uid)
        })
    }

    fn vals_mut<T: ErasedTy>(&mut self) -> Result<&mut Vec<T>, Error> {
        let hint = self.hint().clone();
        let action = *self.action();
        let uid = self.uid();

        self.accessor_mut().vals_mut().map_err(|e| {
            e.cause(raise_error!(
                "Can not find value(mut) of `{}`({})",
                hint,
                action
            ))
            .with_uid(uid)
        })
    }

    fn entry<T: ErasedTy>(&mut self) -> Entry<'_, Vec<T>> {
        self.accessor_mut().entry::<T>()
    }

    fn rawval(&self) -> Result<&RawVal, Error> {
        let uid = self.uid();

        self.accessor().rawval().map_err(|e| {
            e.cause(raise_error!(
                "Can not find raw value(ref) of `{}`",
                self.hint(),
            ))
            .with_uid(uid)
        })
    }

    fn rawval_mut(&mut self) -> Result<&mut RawVal, Error> {
        let hint = self.hint().clone();
        let uid = self.uid();

        self.accessor_mut().rawval_mut().map_err(|e| {
            e.cause(raise_error!("Can not find raw value(mut) of `{}`", hint))
                .with_uid(uid)
        })
    }

    fn rawvals(&self) -> Result<&Vec<RawVal>, Error> {
        self.accessor().rawvals().map_err(|e| {
            e.cause(raise_error!(
                "Can not find raw values(ref) of `{}`",
                self.hint(),
            ))
        })
    }

    fn rawvals_mut(&mut self) -> Result<&mut Vec<RawVal>, Error> {
        let hint = self.hint().clone();
        let uid = self.uid();

        self.accessor_mut().rawvals_mut().map_err(|e| {
            e.cause(raise_error!("Can not find raw values(mut) of `{}`", hint,))
                .with_uid(uid)
        })
    }

    /// Filter the value from option values if `f` return true.
    fn filter<T: ErasedTy>(&mut self, mut f: impl FnMut(&T) -> bool) -> Result<Vec<T>, Error> {
        let vals = self.vals_mut::<T>()?;
        let mut i = 0;
        let mut removed = vec![];

        while i < vals.len() {
            if (f)(&vals[i]) {
                removed.push(vals.remove(i));
            } else {
                i += 1;
            }
        }
        Ok(removed)
    }
}
