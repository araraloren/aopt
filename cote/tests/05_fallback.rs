use cote::prelude::*;
use std::sync::OnceLock;

#[derive(Debug, Cote)]
#[cote(fallback = cli_main, then = storer)]
pub struct Cli;

static FLAG: OnceLock<bool> = OnceLock::new();

fn cli_main<Set, Ser>(_: &mut Set, _: &mut Ser) -> cote::Result<Option<()>> {
    FLAG.get_or_init(|| true);
    Ok(None)
}

fn storer<Set, Ser>(
    _: Uid,
    _: &mut Set,
    _: &mut Ser,
    _: Option<&RawVal>,
    _: Option<()>,
) -> cote::Result<bool> {
    unreachable!("not call here if cli_main returns None")
}

#[test]
fn fallback() {
    assert!(fallback_impl().is_ok());
}

fn fallback_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let CoteRes { parser, .. } = Cli::parse_args(Args::from(["app"].into_iter()))?;
    assert_eq!(FLAG.get(), Some(&true), "Set flag in cli_main");
    assert_eq!(
        parser.find_opt("".infer::<Main>())?.rawval()?,
        &RawVal::from("app")
    );
    Ok(())
}
