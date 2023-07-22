use std::sync::OnceLock;
use cote::*;

#[derive(Debug, Cote)]
#[cote(on = cli_main)]
pub struct Cli;

static FLAG: OnceLock<bool> = OnceLock::new();

fn cli_main<Set, Ser>(set: &mut Set, _: &mut Ser) -> Result<Option<()>, aopt::Error>
where
    Set: cote::Set,
{
    FLAG.get_or_init(|| true);
    assert_eq!(set.len(), 1, "there is only one option here");
    Ok(Some(()))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse(Args::from(["app"].into_iter()))?;
    assert_eq!(FLAG.get(), Some(&true), "Set flag in cli_main");
    Ok(())
}