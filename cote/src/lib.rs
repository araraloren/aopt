pub mod ctx;
pub mod meta;
pub mod services;
pub mod valid;
pub mod value;

use std::ops::{Deref, DerefMut};

use aopt::prelude::*;
use aopt::value::Placeholder;
use aopt::Error;
use aopt::RawVal;

use crate::meta::IntoConfig;

pub trait IntoParserDerive<Set, Inv, Ser>
where
    Set: aopt::prelude::Set + Default,
    SetCfg<Set>: Config + ConfigValue + Default,
{
    fn into_parser(set: Set, inv: Inv, ser: Ser) -> Result<Parser<Set, Inv, Ser>, Error> {
        let mut parser = Parser::new(set, inv, ser);
        Self::update(&mut parser)?;
        Ok(parser)
    }

    fn into_parser_with<'a, P>(policy: &P) -> Result<Parser<Set, Inv, Ser>, Error>
    where
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser> + APolicyExt<P>,
    {
        let mut parser = Parser::new_with(policy);
        Self::update(&mut parser)?;
        Ok(parser)
    }

    fn update(parser: &mut Parser<Set, Inv, Ser>) -> Result<(), Error>;
}

pub trait ExtractFromSetDerive<'a, S>
where
    S: SetValueFindExt,
{
    fn try_extract(set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: Sized;
}

#[derive(Debug, Default)]
pub struct CoteParser<Set, Inv, Ser> {
    name: String,

    parser: Parser<Set, Inv, Ser>,
}

impl<Set, Inv, Ser> Deref for CoteParser<Set, Inv, Ser> {
    type Target = Parser<Set, Inv, Ser>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<Set, Inv, Ser> DerefMut for CoteParser<Set, Inv, Ser> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl<Set, Inv, Ser> CoteParser<Set, Inv, Ser> {
    pub fn new(name: String, set: Set, inv: Inv, ser: Ser) -> Self {
        Self {
            name,
            parser: Parser::new(set, inv, ser),
        }
    }

    pub fn new_with<'a, P>(name: String, policy: &P) -> Self
    where
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser> + APolicyExt<P>,
    {
        Self {
            name,
            parser: Parser::new_with(policy),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn inner_parser(&self) -> &Parser<Set, Inv, Ser> {
        &self.parser
    }

    pub fn inner_parser_mut(&mut self) -> &mut Parser<Set, Inv, Ser> {
        &mut self.parser
    }
}

impl<'a, Set, Inv, Ser> CoteParser<Set, Inv, Ser>
where
    SetOpt<Set>: Opt,
    Set: aopt::set::Set + OptValidator + OptParser,
    <Set as OptParser>::Output: Information,
    SetCfg<Set>: Config + ConfigValue + Default,
    Inv: HandlerCollection<'a, Set, Ser>,
{
    pub fn add_opt_meta(
        &mut self,
        meta: impl IntoConfig<Ret = SetCfg<Set>>,
    ) -> Result<ParserCommit<'a, '_, Inv, Set, Ser, Placeholder>, aopt::Error> {
        let set = self.parser.optset();
        let config = meta.into_config(set)?;

        self.parser.add_opt_cfg(config)
    }

    /// This function will insert help option `--help;-h;-?: Display help message`.
    pub fn add_help_option(&mut self) -> Result<&mut Self, aopt::Error> {
        self.add_opt_i::<bool>("--help;-h;-?: Display help message")?;
        Ok(self)
    }
}

#[derive(Debug)]
pub struct CoteApp<'a, P>
where
    P: Policy,
{
    author: String,

    version: String,

    description: String,

    parser: CoteParser<P::Set, P::Inv<'a>, P::Ser>,
}

impl<'a, P: Policy> Deref for CoteApp<'a, P> {
    type Target = CoteParser<P::Set, P::Inv<'a>, P::Ser>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<'a, P: Policy> DerefMut for CoteApp<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}