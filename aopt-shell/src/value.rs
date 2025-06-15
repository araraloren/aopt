use std::ffi::OsStr;

use crate::acore::Error;

pub trait Values<O> {
    type Err: Into<Error>;

    fn get_values(&mut self, opt: &O) -> Result<&[&OsStr], Self::Err>;
}

impl<O> Values<O> for [&OsStr] {
    type Err = Error;

    fn get_values(&mut self, _: &O) -> Result<&[&OsStr], Self::Err> {
        Ok(self)
    }
}

impl<const N: usize, O> Values<O> for [&OsStr; N] {
    type Err = Error;

    fn get_values(&mut self, _: &O) -> Result<&[&OsStr], Self::Err> {
        Ok(self)
    }
}

impl<O> Values<O> for Vec<&OsStr> {
    type Err = Error;

    fn get_values(&mut self, opt: &O) -> Result<&[&OsStr], Self::Err> {
        Values::get_values(self.as_mut_slice(), opt)
    }
}

pub fn wrap<O, T: Values<O>>(inner: T) -> Adapter<T> {
    Adapter { inner }
}

pub struct Adapter<T> {
    pub inner: T,
}

impl<O, T: Values<O>> Values<O> for Adapter<T> {
    type Err = Error;

    fn get_values(&mut self, opt: &O) -> Result<&[&OsStr], Self::Err> {
        self.inner.get_values(opt).map_err(Into::into)
    }
}
