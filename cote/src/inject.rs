use aopt::prelude::*;
use aopt::Error;
use aopt::GetoptRes;

pub trait CoteParserDeriveExt<P>
where
    P::Set: Set,
    P::Error: Into<aopt::Error>,
    P: Policy + APolicyExt<P> + Default,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    fn parse_env<'zlifetime>() -> Result<GetoptRes<P::Ret, Parser<'zlifetime, P>>, Error> {
        Self::parse(ARef::new(Args::from_env()))
    }

    fn parse<'zlifetime>(
        args: ARef<Args>,
    ) -> Result<GetoptRes<P::Ret, Parser<'zlifetime, P>>, Error> {
        let mut parser = Self::into_parser()?;
        let ret = parser.parse(args).map_err(Into::into)?;

        Ok(GetoptRes {
            ret: ret,
            parser: parser,
        })
    }

    fn into_parser<'zlifetime>() -> Result<Parser<'zlifetime, P>, Error> {
        let mut parser = Parser::<'zlifetime, P>::new(P::default());

        Self::update(&mut parser)?;
        Ok(parser)
    }

    fn update<'zlifetime>(parser: &mut Parser<'zlifetime, P>) -> Result<(), Error>;
}

pub trait CoteParserExtractValueExt<'zlifetime, S>
where
    S: SetValueFindExt,
{
    fn try_extract(set: &'zlifetime mut S) -> Result<Self, Error>
    where
        Self: Sized;
}
