use crate::err::Error;
use crate::Str;

pub trait Creator {
    type Opt;
    type Config;
    type Error: Into<Error>;

    fn get_type_name(&self) -> Str;

    fn is_support_deactivate_style(&self) -> bool;

    fn create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error>;
}
