use crate::err::Error;
use crate::opt::Creator;
use crate::Str;

pub trait ACreator {
    type Opt;
    type Config;

    fn _get_type_name(&self) -> Str;

    fn _support_deactivate_style(&self) -> bool;

    fn _create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Error>;
}

impl<Opt, Config> std::fmt::Debug for Box<dyn ACreator<Opt = Opt, Config = Config>>
where
    Opt: crate::opt::Opt,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Box")
            .field(&format!("Creator({})", self.get_type_name()))
            .finish()
    }
}

impl<Opt, Config> Creator for Box<dyn ACreator<Opt = Opt, Config = Config>>
where
    Opt: crate::opt::Opt,
{
    type Opt = Opt;

    type Config = Config;

    type Error = Error;

    fn get_type_name(&self) -> Str {
        self._get_type_name()
    }

    fn is_support_deactivate_style(&self) -> bool {
        self._support_deactivate_style()
    }

    fn create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        self._create_with(config)
    }
}
