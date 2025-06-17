use std::cell::RefCell;
use std::ffi::OsStr;
use std::ffi::OsString;

use crate::acore::Error;

pub trait Values<O> {
    type Err: Into<Error>;

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err>;
}

impl<O> Values<O> for [&OsStr] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.iter().map(OsString::from).collect())
    }
}

impl<O> Values<O> for [OsString] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.to_vec())
    }
}

impl<O> Values<O> for [&str] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.iter().map(OsString::from).collect())
    }
}

impl<O> Values<O> for [String] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.iter().map(OsString::from).collect())
    }
}

impl<const N: usize, O> Values<O> for [&OsStr; N] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.iter().map(OsString::from).collect())
    }
}

impl<const N: usize, O> Values<O> for [OsString; N] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.to_vec())
    }
}

impl<const N: usize, O> Values<O> for [&str; N] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.iter().map(OsString::from).collect())
    }
}

impl<const N: usize, O> Values<O> for [String; N] {
    type Err = Error;

    fn get_values(&self, _: &O) -> Result<Vec<OsString>, Self::Err> {
        Ok(self.iter().map(OsString::from).collect())
    }
}

impl<O> Values<O> for Vec<&OsStr> {
    type Err = Error;

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err> {
        Values::get_values(self.as_slice(), opt)
    }
}

impl<O> Values<O> for Vec<OsString> {
    type Err = Error;

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err> {
        Values::get_values(self.as_slice(), opt)
    }
}

impl<O> Values<O> for Vec<&str> {
    type Err = Error;

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err> {
        Values::get_values(self.as_slice(), opt)
    }
}

impl<O> Values<O> for Vec<String> {
    type Err = Error;

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err> {
        Values::get_values(self.as_slice(), opt)
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

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err> {
        self.inner.get_values(opt).map_err(Into::into)
    }
}

/// Calls the given function and initialize the value upon first use.
pub struct OnceValues<O> {
    vals: RefCell<Option<Vec<OsString>>>,

    #[allow(clippy::type_complexity)]
    handler: Box<dyn Fn(&O) -> Result<Vec<OsString>, Error>>,
}

impl<O> OnceValues<O> {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&O) -> Result<Vec<OsString>, Error> + 'static,
    {
        Self {
            vals: RefCell::new(None),
            handler: Box::new(handler),
        }
    }
}

/// Calls the given function and initialize the value upon first use.
pub fn once_values<O, F>(handler: F) -> OnceValues<O>
where
    F: Fn(&O) -> Result<Vec<OsString>, Error> + 'static,
{
    OnceValues::new(handler)
}

impl<O> Values<O> for OnceValues<O> {
    type Err = Error;

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err> {
        if self.vals.borrow().is_none() {
            *self.vals.borrow_mut() = Some((self.handler)(opt)?);
        }
        self.vals
            .borrow()
            .clone()
            .ok_or_else(|| crate::error!("can not get values in OnceValues"))
    }
}

/// Calls the given function and initializes the value each time it is used.
pub struct RepeatValues<O> {
    #[allow(clippy::type_complexity)]
    handler: Box<dyn Fn(&O) -> Result<Vec<OsString>, Error>>,
}

impl<O> RepeatValues<O> {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&O) -> Result<Vec<OsString>, Error> + 'static,
    {
        Self {
            handler: Box::new(handler),
        }
    }
}

/// Calls the given function and initializes the value each time it is used.
pub fn repeat_values<O, F>(handler: F) -> RepeatValues<O>
where
    F: Fn(&O) -> Result<Vec<OsString>, Error> + 'static,
{
    RepeatValues::new(handler)
}

impl<O> Values<O> for RepeatValues<O> {
    type Err = Error;

    fn get_values(&self, opt: &O) -> Result<Vec<OsString>, Self::Err> {
        (self.handler)(opt)
    }
}
