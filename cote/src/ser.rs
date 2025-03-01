use crate::Result;
use aopt::map::ErasedTy;
use aopt::prelude::ASer;
use aopt::raise_error;
use aopt::ser::ServicesValExt;

pub trait AppStorage {
    fn app_data<T: ErasedTy>(&self) -> Result<&T>;

    fn app_data_mut<T: ErasedTy>(&mut self) -> Result<&mut T>;

    fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Option<T>;

    fn take_app_data<T: ErasedTy>(&mut self) -> Result<T>;
}

#[doc(hidden)]
pub trait ASerTransfer {
    fn transfer_app_ser_to(&mut self, other: &mut Self) -> Result<()>;

    fn set_app_ser(&mut self, ser: aopt::prelude::ASer) -> Result<()>;
}

#[derive(Debug)]
pub struct CoteSer {
    ser: Option<ASer>,
    app_ser: Option<ASer>,
}

impl Default for CoteSer {
    fn default() -> Self {
        Self {
            ser: Some(ASer::default()),
            app_ser: Some(ASer::default()),
        }
    }
}

impl CoteSer {
    fn ser_mut(&mut self) -> Result<&mut ASer> {
        self.ser
            .as_mut()
            .ok_or_else(|| raise_error!("can not find ser, maybe a bug?"))
    }

    fn ser(&self) -> Result<&ASer> {
        self.ser
            .as_ref()
            .ok_or_else(|| raise_error!("can not find ser, maybe a bug?"))
    }

    fn app_ser_mut(&mut self) -> Result<&mut ASer> {
        self.app_ser
            .as_mut()
            .ok_or_else(|| raise_error!("can not find app data ser, maybe a bug?"))
    }

    fn app_ser(&self) -> Result<&ASer> {
        self.app_ser
            .as_ref()
            .ok_or_else(|| raise_error!("can not find app data ser, maybe a bug?"))
    }
}

impl AppStorage for CoteSer {
    fn app_data<T: ErasedTy>(&self) -> Result<&T> {
        self.app_ser()?.sve_val()
    }

    fn app_data_mut<T: ErasedTy>(&mut self) -> Result<&mut T> {
        self.app_ser_mut()?.sve_val_mut()
    }

    fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Option<T> {
        self.app_ser_mut().ok().and_then(|v| v.sve_insert(val))
    }

    fn take_app_data<T: ErasedTy>(&mut self) -> Result<T> {
        self.app_ser_mut()?.sve_take_val()
    }
}

impl ServicesValExt for CoteSer {
    fn sve_insert<T: ErasedTy>(&mut self, val: T) -> Option<T> {
        ServicesValExt::sve_insert(self.ser_mut().unwrap(), val)
    }

    fn sve_val<T: ErasedTy>(&self) -> std::result::Result<&T, aopt::Error> {
        ServicesValExt::sve_val(self.ser().unwrap())
    }

    fn sve_val_mut<T: ErasedTy>(&mut self) -> std::result::Result<&mut T, aopt::Error> {
        ServicesValExt::sve_val_mut(self.ser_mut().unwrap())
    }

    fn sve_take_val<T: ErasedTy>(&mut self) -> std::result::Result<T, aopt::Error> {
        ServicesValExt::sve_take_val(self.ser_mut().unwrap())
    }
}

impl ASerTransfer for CoteSer {
    fn transfer_app_ser_to(&mut self, other: &mut Self) -> Result<()> {
        other.set_app_ser(
            self.app_ser
                .take()
                .ok_or_else(|| raise_error!("can not find app ser, maybe a bug?"))?,
        )
    }

    fn set_app_ser(&mut self, ser: aopt::prelude::ASer) -> Result<()> {
        self.app_ser = Some(ser);
        Ok(())
    }
}
