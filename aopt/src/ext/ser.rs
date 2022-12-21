//! The structs hold the data collect from [`Services`](crate::ser::Services).
//! They are all implemented [`Extract`].
use crate::ext::ServicesExt;
use crate::ext::ServicesValExt;
use crate::map::ErasedTy;
use crate::prelude::CheckService;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::UsrValService;
use crate::ser::ValService;
use crate::set::Set;
use crate::Error;
use crate::RawVal;
use crate::Uid;

impl ServicesExt for Services {
    fn ser_val(&self) -> Result<&ValService, Error> {
        self.service::<ValService>()
    }

    fn ser_val_mut(&mut self) -> Result<&mut ValService, Error> {
        self.service_mut::<ValService>()
    }

    fn ser_usrval(&self) -> Result<&UsrValService, Error> {
        self.service::<UsrValService>()
    }

    fn ser_usrval_mut(&mut self) -> Result<&mut UsrValService, Error> {
        self.service_mut::<UsrValService>()
    }

    fn ser_invoke<S: Set + 'static>(&self) -> Result<&InvokeService<S>, Error> {
        self.service::<InvokeService<S>>()
    }

    fn ser_invoke_mut<S: Set + 'static>(&mut self) -> Result<&mut InvokeService<S>, Error> {
        self.service_mut::<InvokeService<S>>()
    }

    fn ser_rawval<T: ErasedTy>(&self) -> Result<&RawValService<T>, Error> {
        self.service::<RawValService<T>>()
    }

    fn ser_rawval_mut<T: ErasedTy>(&mut self) -> Result<&mut RawValService<T>, Error> {
        self.service_mut::<RawValService<T>>()
    }

    fn ser_check<S: Set + 'static>(&self) -> Result<&CheckService<S>, Error> {
        self.service::<CheckService<S>>()
    }
}

impl ServicesValExt for Services {
    fn sve_val<T: ErasedTy>(&self, uid: Uid) -> Result<&T, Error> {
        self.ser_val()?.val(uid)
    }

    fn sve_val_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut T, Error> {
        self.ser_val_mut()?.val_mut(uid)
    }

    fn sve_take_val<T: ErasedTy>(&mut self, uid: Uid) -> Result<T, Error> {
        self.ser_val_mut()?
            .pop(uid)
            .ok_or_else(|| Error::raise_error("Can not take value from ValService"))
    }

    fn sve_vals<T: ErasedTy>(&self, uid: Uid) -> Result<&Vec<T>, Error> {
        self.ser_val()?.vals(uid)
    }

    fn sve_vals_mut<T: ErasedTy>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error> {
        self.ser_val_mut()?.vals_mut(uid)
    }

    fn sve_take_vals<T: ErasedTy>(&mut self, uid: Uid) -> Result<Vec<T>, Error> {
        self.ser_val_mut()?
            .remove(uid)
            .ok_or_else(|| Error::raise_error("Can not take values from ValService"))
    }

    fn sve_filter<T: ErasedTy>(
        &mut self,
        uid: Uid,
        mut f: impl FnMut(&T) -> bool,
    ) -> Result<Vec<T>, Error> {
        let vals = self.sve_vals_mut::<T>(uid)?;
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

    fn sve_usrval<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.ser_usrval()?.val::<T>()
    }

    fn sve_usrval_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.ser_usrval_mut()?.val_mut::<T>()
    }

    fn sve_take_usrval<T: ErasedTy>(&mut self) -> Result<T, Error> {
        self.ser_usrval_mut()?
            .remove::<T>()
            .ok_or_else(|| Error::raise_error("Can not take value from UsrValService"))
    }

    /// Get the raw value reference of option `uid` from [`RawValService`].
    fn sve_rawval(&self, uid: Uid) -> Result<&RawVal, Error> {
        self.ser_rawval()?.val(uid)
    }

    /// Get the raw value mutable reference of option `uid` from [`RawValService`].
    fn sve_rawval_mut(&mut self, uid: Uid) -> Result<&mut RawVal, Error> {
        self.ser_rawval_mut()?.val_mut(uid)
    }

    /// Get the raw values reference of option `uid` from [`RawValService`].
    fn sve_rawvals(&self, uid: Uid) -> Result<&Vec<RawVal>, Error> {
        self.ser_rawval()?.vals(uid)
    }

    /// Get the raw values mutable reference of option `uid` from [`RawValService`].
    fn sve_rawvals_mut(&mut self, uid: Uid) -> Result<&mut Vec<RawVal>, Error> {
        self.ser_rawval_mut()?.vals_mut(uid)
    }
}
