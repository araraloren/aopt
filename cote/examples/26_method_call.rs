use cote::aopt::value::ValidatorHandler;
use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(parser_mod(parser))]
pub struct Cli {
    // `bar` has a index 2
    #[pos(index = 1, bar_storer(cfg, Box::new(|val: &usize| *val > 32)))]
    bar: usize,
}

fn bar_storer<S: Set>(cfg: &mut cote::prelude::SetCfg<S>, func: ValidatorHandler<usize>)
where
    cote::prelude::SetCfg<S>: cote::prelude::ConfigValue,
{
    cfg.set_storer(ValStorer::new_validator(ValValidator::new(Box::new(func))));
}

fn parser_mod<S>(_parser: &mut Parser<'_, S>) -> Result<(), cote::Error> {
    Ok(())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    assert!(Cli::parse(Args::from(["app", "18"])).is_err());
    assert!(Cli::parse(Args::from(["app", "34"])).is_ok());

    Ok(())
}
