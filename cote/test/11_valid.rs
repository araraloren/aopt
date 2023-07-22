use cote::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    #[allow(unused)]
    #[pos(valid = valid!(["lily", "lucy", "bob", "joe"]))]
    name: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    assert!(Cli::parse(Args::from(["app", "lily"].into_iter())).is_ok());
    assert!(Cli::parse(Args::from(["app", "jim"].into_iter())).is_err());
    Ok(())
}
