use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    #[allow(unused)]
    #[pos(valid = valid!(["lily", "lucy", "bob", "joe"]))]
    name: String,
}

#[test]
fn valid() {
    assert!(valid_impl().is_ok());
}

fn valid_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    assert!(Cli::parse(Args::from(["app", "lily"].into_iter())).is_ok());
    assert!(Cli::parse(Args::from(["app", "jim"].into_iter())).is_err());
    Ok(())
}
