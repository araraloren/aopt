use std::fmt::Write;
use std::marker::PhantomData;

use crate::acore::Error;
use crate::acore::opt::Opt;
use crate::shell::Shell;

pub struct Zsh<O, W> {
    w: Option<W>,
    __marker: PhantomData<O>,
}

impl<O, W> Default for Zsh<O, W> {
    fn default() -> Self {
        Self {
            w: Default::default(),
            __marker: Default::default(),
        }
    }
}

impl<O, W> Zsh<O, W> {
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

impl<O, W> Shell<O, W> for Zsh<O, W>
where
    W: Write,
    O: Opt,
{
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        name == "zsh"
    }

    fn set_buff(&mut self, w: W) {
        self.w = Some(w);
    }

    fn write_cmd(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        if opt.help().is_empty() {
            wln2buf!(self.buffer()?, "{}", name)
        } else {
            wln2buf!(self.buffer()?, "{}:{}", name, opt.help())
        }
    }

    fn write_opt(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        if opt.help().is_empty() {
            wln2buf!(self.buffer()?, "{}", name)
        } else {
            wln2buf!(self.buffer()?, "{}:{}", name, opt.help())
        }
    }

    fn write_pos(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        if opt.help().is_empty() {
            wln2buf!(self.buffer()?, "{}", name)
        } else {
            wln2buf!(self.buffer()?, "{}:{}", name, opt.help())
        }
    }

    fn write_val(&mut self, val: &std::ffi::OsStr, _: &O) -> Result<(), Self::Err> {
        wln2buf!(self.buffer()?, "{}", val.display())
    }

    fn write_eq(&mut self, name: &str, val: &std::ffi::OsStr, _: &O) -> Result<(), Self::Err> {
        wln2buf!(self.buffer()?, "{}={}", name, val.display())
    }

    fn finish(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn take_buff(&mut self) -> Option<W> {
        self.w.take()
    }
}
