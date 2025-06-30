pub(crate) mod bash;
pub(crate) mod fish;
pub(crate) mod ps1;
pub(crate) mod zsh;

use std::ffi::OsStr;
use std::io::Write;

use crate::acore::opt::Opt;
use crate::acore::opt::Style;
use crate::acore::trace;
use crate::acore::HashMap;
use crate::acore::Uid;
use crate::value::Values;
use crate::Error;

pub use bash::Bash;
pub use fish::Fish;
pub use ps1::PowerShell;
pub use zsh::Zsh;

pub type PowerShell7<O, W> = PowerShell<O, W>;

pub trait Complete<O> {
    type Out;
    type Ctx<'a>;
    type Err: Into<Error>;

    fn complete<'a, T, W>(
        &self,
        s: &mut T,
        ctx: &mut Self::Ctx<'a>,
    ) -> Result<Self::Out, Self::Err>
    where
        T: Shell<O, W>;
}

macro_rules! name_iter {
    ($opt:ident) => {
        std::iter::once($opt.name()).chain(
            $opt.alias()
                .iter()
                .flat_map(|v| v.iter().map(|v| v.as_str())),
        )
    };
}

pub fn complete_cmd<'a, O, I, F>(arg: &str, opts: I, mut f: F) -> Result<bool, Error>
where
    O: Opt + 'a,
    I: Iterator<Item = &'a O>,
    F: FnMut(&str, &O) -> Result<(), Error>,
{
    let mut found = false;

    for opt in opts.filter(|v| v.mat_style(Style::Cmd)) {
        for name in name_iter!(opt).filter(|v| v.starts_with(arg)) {
            trace!("available cmd -> {name}");
            f(name, opt)?;
            found = true;
        }
    }
    Ok(found)
}

pub fn complete_val<'a, O, I, F>(
    arg: &str,
    bytes: &[u8],
    opts: I,
    values: &HashMap<Uid, Box<dyn Values<O, Err = Error>>>,
    mut f: F,
) -> Result<bool, Error>
where
    O: Opt + 'a,
    I: Iterator<Item = &'a O>,
    F: FnMut(&OsStr, &O) -> Result<(), Error>,
{
    complete_eq(arg, bytes, opts, values, |_, val, opt| f(val, opt))
}

pub fn complete_eq<'a, O, I, F>(
    arg: &str,
    bytes: &[u8],
    opts: I,
    values: &HashMap<Uid, Box<dyn Values<O, Err = Error>>>,
    mut f: F,
) -> Result<bool, Error>
where
    O: Opt + 'a,
    I: Iterator<Item = &'a O>,
    F: FnMut(&str, &OsStr, &O) -> Result<(), Error>,
{
    let mut found = false;

    for opt in opts.filter(|v| v.mat_style(Style::Argument)) {
        for name in name_iter!(opt).filter(|v| v == &arg) {
            if name == arg {
                if let Some(getter) = values.get(&opt.uid()) {
                    for val in getter.get_values(opt)? {
                        if !val.is_empty() && bytes.is_empty()
                            || bytes
                                .iter()
                                .zip(val.as_encoded_bytes())
                                .all(|(a, b)| *a == *b)
                        {
                            trace!("available opt value -> {}", val.display());
                            f(arg, &val, opt)?;
                            found = true;
                        }
                    }
                }
                break;
            }
        }
    }
    Ok(found)
}

pub fn complete_opt<'a, O, I, F>(arg: &str, opts: I, mut f: F) -> Result<bool, Error>
where
    O: Opt + 'a,
    I: Iterator<Item = &'a O>,
    F: FnMut(&str, &O) -> Result<(), Error>,
{
    let mut found = false;

    for opt in opts.filter(|v| {
        v.mat_style(Style::Argument)
            || v.mat_style(Style::Boolean)
            || v.mat_style(Style::Combined)
            || v.mat_style(Style::Flag)
    }) {
        for name in name_iter!(opt).filter(|v| v.starts_with(arg)) {
            trace!("available opt -> {name}");
            f(name, opt)?;
            found = true;
        }
    }
    Ok(found)
}

pub trait Shell<O, W> {
    type Err: Into<Error>;

    fn is_avail(&self, name: &str) -> bool;

    fn set_buff(&mut self, w: W);

    fn write_cmd(&mut self, name: &str, opt: &O) -> Result<(), Self::Err>;

    fn write_opt(&mut self, name: &str, opt: &O) -> Result<(), Self::Err>;

    fn write_pos(&mut self, name: &str, opt: &O) -> Result<(), Self::Err>;

    fn write_val(&mut self, val: &OsStr, opt: &O) -> Result<(), Self::Err>;

    fn write_eq(&mut self, name: &str, val: &OsStr, opt: &O) -> Result<(), Self::Err>;

    fn finish(&mut self) -> Result<(), Self::Err>;

    fn take_buff(&mut self) -> Option<W>;
}

impl<O, W, E: Into<Error>> Shell<O, W> for Box<dyn Shell<O, W, Err = E>> {
    type Err = E;

    fn is_avail(&self, name: &str) -> bool {
        Shell::is_avail(self.as_ref(), name)
    }

    fn write_cmd(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        Shell::write_cmd(self.as_mut(), name, opt)
    }

    fn write_opt(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        Shell::write_opt(self.as_mut(), name, opt)
    }

    fn write_pos(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        Shell::write_pos(self.as_mut(), name, opt)
    }

    fn write_val(&mut self, val: &OsStr, opt: &O) -> Result<(), Self::Err> {
        Shell::write_val(self.as_mut(), val, opt)
    }

    fn write_eq(&mut self, name: &str, val: &OsStr, opt: &O) -> Result<(), Self::Err> {
        Shell::write_eq(self.as_mut(), name, val, opt)
    }

    fn set_buff(&mut self, w: W) {
        Shell::set_buff(self.as_mut(), w);
    }

    fn finish(&mut self) -> Result<(), Self::Err> {
        Shell::finish(self.as_mut())
    }

    fn take_buff(&mut self) -> Option<W> {
        Shell::take_buff(self.as_mut())
    }
}

pub fn wrap<O, W, S: Shell<O, W>>(inner: S) -> Adapter<S> {
    Adapter { inner }
}

pub fn wrapref<'a, O, W, S: Shell<O, W>>(inner: &'a mut S) -> AdapterRef<'a, S> {
    AdapterRef { inner }
}

pub struct Adapter<T> {
    pub inner: T,
}

impl<O, W, T: Shell<O, W>> Shell<O, W> for Adapter<T> {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        self.inner.is_avail(name)
    }

    fn write_cmd(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_cmd(name, opt).map_err(Into::into)
    }

    fn write_opt(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_opt(name, opt).map_err(Into::into)
    }

    fn write_pos(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_pos(name, opt).map_err(Into::into)
    }

    fn write_val(&mut self, val: &OsStr, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_val(val, opt).map_err(Into::into)
    }

    fn write_eq(&mut self, name: &str, val: &OsStr, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_eq(name, val, opt).map_err(Into::into)
    }

    fn set_buff(&mut self, w: W) {
        self.inner.set_buff(w);
    }

    fn finish(&mut self) -> Result<(), Self::Err> {
        self.inner.finish().map_err(Into::into)
    }

    fn take_buff(&mut self) -> Option<W> {
        self.inner.take_buff()
    }
}

pub struct AdapterRef<'a, T> {
    pub inner: &'a mut T,
}

impl<'a, O, W, T: Shell<O, W>> Shell<O, W> for AdapterRef<'a, T> {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        self.inner.is_avail(name)
    }

    fn write_cmd(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_cmd(name, opt).map_err(Into::into)
    }

    fn write_opt(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_opt(name, opt).map_err(Into::into)
    }

    fn write_pos(&mut self, name: &str, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_pos(name, opt).map_err(Into::into)
    }

    fn write_val(&mut self, val: &OsStr, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_val(val, opt).map_err(Into::into)
    }

    fn write_eq(&mut self, name: &str, val: &OsStr, opt: &O) -> Result<(), Self::Err> {
        self.inner.write_eq(name, val, opt).map_err(Into::into)
    }

    fn set_buff(&mut self, w: W) {
        self.inner.set_buff(w);
    }

    fn finish(&mut self) -> Result<(), Self::Err> {
        self.inner.finish().map_err(Into::into)
    }

    fn take_buff(&mut self) -> Option<W> {
        self.inner.take_buff()
    }
}

pub struct Manager<'a, O, W> {
    gens: Vec<Box<dyn Shell<O, W, Err = Error> + 'a>>,
}

impl<'a, O, W> Default for Manager<'a, O, W>
where
    W: Write + 'a,
    O: Opt + 'a,
{
    fn default() -> Self {
        Self {
            gens: vec![
                Box::new(Bash::new()),
                Box::new(Fish::new()),
                Box::new(PowerShell::new()),
                Box::new(Zsh::new()),
                Box::new(PowerShell7::new7()),
            ],
        }
    }
}

impl<'a, O: 'a, W: 'a> Manager<'a, O, W> {
    pub fn new() -> Self {
        Self { gens: vec![] }
    }

    pub fn register(&mut self, r#gen: impl Shell<O, W> + 'a) -> &mut Self {
        self.gens.push(Box::new(wrap(r#gen)));
        self
    }

    pub fn find(&self, shell: &str) -> Result<&(dyn Shell<O, W, Err = Error>), Error> {
        self.gens
            .iter()
            .find(|v| v.as_ref().is_avail(shell))
            .ok_or_else(|| crate::error!("can not find shell type `{shell}`"))
            .map(|v| v.as_ref())
    }

    pub fn find_mut(
        &mut self,
        shell: &str,
    ) -> Result<&mut Box<dyn Shell<O, W, Err = Error> + 'a>, Error> {
        self.gens
            .iter_mut()
            .find(|v| v.as_ref().is_avail(shell))
            .ok_or_else(|| crate::error!("can not find shell type `{shell}`"))
    }

    pub fn take(&mut self, shell: &str) -> Result<Box<dyn Shell<O, W, Err = Error> + 'a>, Error> {
        self.gens
            .iter()
            .position(|v| v.as_ref().is_avail(shell))
            .ok_or_else(|| crate::error!("can not find shell type `{shell}`"))
            .map(|v| self.gens.swap_remove(v))
    }
}
