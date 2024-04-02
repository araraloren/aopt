use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    #[allow(unused)]
    #[pos(force = true)]
    name: Option<String>,
}

#[test]
fn force() {
    assert!(force_impl().is_ok());
}

fn force_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // name is not set, parse failed
    assert!(Cli::parse(Args::from(["app"].into_iter())).is_err());
    Ok(())
}
