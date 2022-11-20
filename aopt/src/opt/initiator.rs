use std::fmt::Debug;

use crate::ext::ServicesExt;
use crate::ser::Services;
use crate::Error;
use crate::Uid;
use tracing::trace;

pub trait ValInitialize<T: 'static> {
    type Error: Into<Error>;

    fn prepare_initialize_val(&mut self) -> Result<T, Self::Error>;
}

impl<Func, Err, T: 'static> ValInitialize<T> for Func
where
    Err: Into<Error>,
    Func: FnMut() -> Result<T, Err>,
{
    type Error = Err;

    fn prepare_initialize_val(&mut self) -> Result<T, Self::Error> {
        (self)()
    }
}

pub type InitiatorHandler = Box<dyn FnMut(Uid, &mut Services) -> Result<(), Error>>;

pub struct ValInitiator(InitiatorHandler);

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        unsafe impl Send for ValInitiator { }

        unsafe impl Sync for ValInitiator { }
    }
}

impl Default for ValInitiator {
    fn default() -> Self {
        Self::null_initiator()
    }
}

impl ValInitiator {
    pub fn new<T: 'static>(mut init: impl ValInitialize<Vec<T>> + 'static) -> Self {
        Self(Box::new(move |uid: Uid, ser: &mut Services| {
            let vals = init.prepare_initialize_val().map_err(|e| e.into())?;
            ser.ser_val_mut()?.set(uid, vals);
            Ok(())
        }))
    }

    pub fn empty<T: 'static>() -> Self {
        Self(Box::new(move |uid: Uid, ser: &mut Services| {
            ser.ser_val_mut()?.set(uid, Vec::<T>::new());
            Ok(())
        }))
    }

    pub fn with<T: Clone + 'static>(initialize_value: Vec<T>) -> Self {
        Self(Box::new(move |uid: Uid, ser: &mut Services| {
            ser.ser_val_mut()?.set(uid, initialize_value.clone());
            Ok(())
        }))
    }

    pub fn do_initialize(&mut self, uid: Uid, ser: &mut Services) -> Result<(), Error> {
        trace!("Try to initialize the value of {{{uid}}}");
        (self.0)(uid, ser)
    }
}

impl Debug for ValInitiator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValInitiator").field(&"{...}").finish()
    }
}

macro_rules! num_initiator {
    ($num:ty, $name:ident) => {
        pub fn $name(val: $num) -> Self {
            Self::new(move || -> Result<Vec<$num>, Error> { Ok(vec![val]) })
        }
    };
}

impl ValInitiator {
    pub fn null_initiator() -> Self {
        Self(Box::new(|_: Uid, _: &mut Services| Ok(())))
    }

    pub fn bool_initiator(val: bool) -> Self {
        Self::new(move || -> Result<Vec<bool>, Error> { Ok(vec![val]) })
    }

    num_initiator!(i8, i8_initiator);

    num_initiator!(i16, i16_initiator);

    num_initiator!(i32, i32_initiator);

    num_initiator!(i64, i64_initiator);

    num_initiator!(u8, u8_initiator);

    num_initiator!(u16, u16_initiator);

    num_initiator!(u32, u32_initiator);

    num_initiator!(u64, u64_initiator);

    num_initiator!(f32, f32_initiator);

    num_initiator!(f64, f64_initiator);
}
