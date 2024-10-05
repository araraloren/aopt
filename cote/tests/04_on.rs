use cote::prelude::*;
use std::sync::OnceLock;

#[derive(Debug, Cote)]
#[cote(on = cli_main)]
pub struct Cli;

static FLAG: OnceLock<bool> = OnceLock::new();

fn cli_main<Set, Ser>(set: &mut Set, _: &mut Ser, _: &Ctx) -> cote::Result<Option<()>>
where
    Set: cote::prelude::Set,
{
    FLAG.get_or_init(|| true);
    assert_eq!(set.len(), 1, "there is only one option here");
    Ok(Some(()))
}

#[test]
fn on() {
    assert!(on_impl().is_ok());
}

fn on_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse(Args::from(["app"].into_iter()))?;
    assert_eq!(FLAG.get(), Some(&true), "Set flag in cli_main");
    Ok(())
}
