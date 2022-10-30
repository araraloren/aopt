pub(crate) mod aopt;
pub(crate) mod config;
pub(crate) mod creator;
pub(crate) mod help;
pub(crate) mod index;
pub(crate) mod info;
pub(crate) mod parser;
pub(crate) mod policy;
pub(crate) mod style;
pub(crate) mod valid;
pub(crate) mod value;

pub use self::aopt::AOpt;
pub use self::config::Config;
pub use self::config::ConfigValue;
pub use self::config::OptConfig;
pub use self::creator::IntCreator;
pub use self::help::Help as OptHelp;
pub use self::index::Index as OptIndex;
pub use self::info::ConstrctInfo;
pub use self::info::Information;
pub use self::parser::StrParser;
pub use self::policy::ValPolicy;
pub use self::policy::ValType;
pub use self::style::Style as OptStyle;
pub use self::valid::RawValValidator;
pub use self::valid::ValValidator;
pub use self::value::RawValParser;

use std::fmt::Debug;

use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

/// Option parser using for parsing option constructor string.
pub trait OptParser {
    type Output;
    type Error: Into<Error>;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error>;
}

pub trait Opt: Debug {
    fn reset(&mut self);

    fn valid(&self) -> bool;

    fn is_deactivate(&self) -> bool;

    fn policy(&self) -> &(ValPolicy, ValType);

    fn hint(&self) -> &Str;

    fn help(&self) -> &Str;

    fn uid(&self) -> Uid;

    fn set_uid(&mut self, uid: Uid);

    fn setted(&self) -> bool;

    fn set_setted(&mut self, setted: bool);

    /// The name of option.
    fn name(&self) -> &Str;

    fn r#type(&self) -> Str;

    /// If the option is optional.
    fn optional(&self) -> bool;

    /// The prefix of option.
    fn prefix(&self) -> Option<&Str>;

    /// The index of option.
    fn idx(&self) -> Option<&OptIndex>;

    /// The alias the option.
    fn alias(&self) -> Option<&Vec<(Str, Str)>>;

    fn mat_style(&self, style: OptStyle) -> bool;

    fn mat_optional(&self, optional: bool) -> bool;

    fn mat_name(&self, name: Option<&Str>) -> bool;

    fn mat_prefix(&self, prefix: Option<&Str>) -> bool;

    fn mat_alias(&self, prefix: &Str, name: &Str) -> bool;

    fn mat_idx(&self, index: Option<(usize, usize)>) -> bool;

    fn check_val(
        &mut self,
        val: Option<&RawVal>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error>;
}

pub trait Creator {
    type Opt;
    type Config;
    type Error: Into<Error>;

    fn r#type(&self) -> Str;

    fn sp_deactivate(&self) -> bool;

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error>;
}

impl<Opt, Config, Err: Into<Error>> Creator
    for Box<dyn Creator<Opt = Opt, Config = Config, Error = Err>>
{
    type Opt = Opt;

    type Config = Config;

    type Error = Err;

    fn r#type(&self) -> Str {
        Creator::r#type(self.as_ref())
    }

    fn sp_deactivate(&self) -> bool {
        Creator::sp_deactivate(self.as_ref())
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        Creator::new_with(self.as_mut(), config)
    }
}

impl<Opt, Config, Err: Into<Error>> Debug
    for Box<dyn Creator<Opt = Opt, Config = Config, Error = Err>>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Creator")
            .field(&format!("{{{}}}", self.r#type()))
            .finish()
    }
}
