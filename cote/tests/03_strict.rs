use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote(help, strict = true)]
pub struct Cli;

#[test]
fn strict() {
    assert!(strict_impl().is_ok());
}

fn strict_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let ret = Cli::parse(Args::from(["app", "--opt-a"].into_iter()));

    assert!(ret.is_err());
    if let Some(err) = ret.err() {
        assert_eq!(err.to_string(), "Parsing arguments `--opt-a` failed: None");
    }
    Ok(())
}
