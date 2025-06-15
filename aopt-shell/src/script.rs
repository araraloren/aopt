pub(crate) mod bash;
pub(crate) mod fish;
pub(crate) mod ps1;
pub(crate) mod zsh;

use std::io::Write;

use crate::Error;
pub use bash::Bash;
pub use fish::Fish;
pub use ps1::PowerShell;
pub use zsh::Zsh;

pub trait Generator {
    type Err: Into<Error>;

    fn is_avail(&self, name: &str) -> bool;

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err>;
}

impl<E: Into<Error>> Generator for Box<dyn Generator<Err = E>> {
    type Err = E;

    fn is_avail(&self, name: &str) -> bool {
        Generator::is_avail(self.as_ref(), name)
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        Generator::generate(self.as_ref(), name, bin)
    }
}

pub fn wrap<T: Generator>(r#gen: T) -> Wrapper<T> {
    Wrapper(r#gen)
}

pub fn wrapref<'a, T: Generator>(r#gen: &'a T) -> WrapperRef<'a, T> {
    WrapperRef(r#gen)
}

pub struct Wrapper<T: Generator>(pub T);

impl<T: Generator> Generator for Wrapper<T> {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        self.0.is_avail(name)
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        self.0.generate(name, bin).map_err(Into::into)
    }
}

pub struct WrapperRef<'a, T: Generator>(pub &'a T);

impl<'a, T: Generator> Generator for WrapperRef<'a, T> {
    type Err = Error;

    fn is_avail(&self, name: &str) -> bool {
        self.0.is_avail(name)
    }

    fn generate(&self, name: &str, bin: &str) -> Result<String, Self::Err> {
        self.0.generate(name, bin).map_err(Into::into)
    }
}

pub struct Manager {
    gens: Vec<Box<dyn Generator<Err = Error>>>,
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            gens: vec![
                Box::new(Bash),
                Box::new(Fish),
                Box::new(PowerShell),
                Box::new(Zsh),
            ],
        }
    }
}

impl Manager {
    pub fn new() -> Self {
        Self { gens: vec![] }
    }

    pub fn register(&mut self, r#gen: impl Generator + 'static) -> &mut Self {
        self.gens.push(Box::new(wrap(r#gen)));
        self
    }

    pub fn find(&self, shell: &str) -> Result<&dyn Generator<Err = Error>, Error> {
        self.gens
            .iter()
            .find(|v| v.is_avail(shell))
            .ok_or_else(|| crate::error!("can not find generator for shell `{shell}`"))
            .map(|v| v.as_ref())
    }

    pub fn find_mut(&mut self, shell: &str) -> Result<&mut Box<dyn Generator<Err = Error>>, Error> {
        self.gens
            .iter_mut()
            .find(|v| v.is_avail(shell))
            .ok_or_else(|| crate::error!("can not find generator for shell `{shell}`"))
    }

    pub fn take(&mut self, shell: &str) -> Result<Box<dyn Generator<Err = Error>>, Error> {
        self.gens
            .iter()
            .position(|v| v.is_avail(shell))
            .ok_or_else(|| crate::error!("can not find generator for shell `{shell}`"))
            .map(|v| self.gens.swap_remove(v))
    }

    pub fn generate(&self, shell: &str, name: &str, bin: &str) -> Result<String, Error> {
        self.find(shell)?.generate(name, bin)
    }

    pub fn write<W>(&self, shell: &str, name: &str, bin: &str, w: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        write!(w, "{}", self.generate(shell, name, bin)?)
            .map_err(|e| crate::error!("can not write script: {e:?}"))?;
        Ok(())
    }
}
