use std::io::Write;
use std::marker::PhantomData;

use crate::acore::Error;
use crate::shell::Shell;

pub struct Bash<O, W> {
    w: Option<W>,
    __marker: PhantomData<O>,
}

impl<O, W> Default for Bash<O, W> {
    fn default() -> Self {
        Self {
            w: Default::default(),
            __marker: Default::default(),
        }
    }
}

impl<O, W> Bash<O, W> {
    pub fn new() -> Self {
        Self {
            w: None,
            __marker: PhantomData,
        }
    }

    pub fn buffer(&mut self) -> Result<&mut W, Error> {
        self.w
            .as_mut()
            .ok_or_else(|| crate::error!("must set buffer before write to"))
    }

    pub fn with_buffer(mut self, w: W) -> Self {
        self.w = Some(w);
        self
    }
}

macro_rules! wln2buf {
    ($w:expr, $fmt:literal, $($arg:tt)*) => {
        writeln!( $w, $fmt, $($arg)* )
            .map_err(|e| $crate::error!("can not write to buffer: {e:?}"))
    };
}

impl<O, W> Shell<O, W> for Bash<O, W>
where
    W: Write,
{
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == "bash"
    }

    fn set_buff(&mut self, w: W) {
        self.w = Some(w);
    }

    fn write_cmd(&mut self, name: &str, _: &O) -> Result<(), Self::Err> {
        wln2buf!(self.buffer()?, "{}", name)
    }

    fn write_opt(&mut self, name: &str, _: &O) -> Result<(), Self::Err> {
        wln2buf!(self.buffer()?, "{}", name)
    }

    fn write_pos(&mut self, name: &str, _: &O) -> Result<(), Self::Err> {
        wln2buf!(self.buffer()?, "{}", name)
    }

    fn write_val(&mut self, val: &std::ffi::OsStr, _: &O) -> Result<(), Self::Err> {
        wln2buf!(self.buffer()?, "{}", val.display())
    }

    fn write_eq(&mut self, _: &str, val: &std::ffi::OsStr, _: &O) -> Result<(), Self::Err> {
        wln2buf!(self.buffer()?, "{}", val.display())
    }

    fn finish(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn take_buff(&mut self) -> Option<W> {
        self.w.take()
    }
}
