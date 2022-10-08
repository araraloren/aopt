use crate::err::Error;
use crate::Str;

pub trait Creator {
    type Opt;
    type Config;
    type Error: Into<Error>;

    fn ty(&self) -> Str;

    fn sp_deact(&self) -> bool;

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error>;
}
