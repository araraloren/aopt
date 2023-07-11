use cote::*;

#[derive(Debug, Cote)]
#[cote(help, strict = true)]
pub struct Cli;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let ret = Cli::parse(Args::from(["app", "--opt-a"].into_iter()));
    
    assert!(ret.is_err());
    if let Some(err) = ret.err() {
        assert_eq!(err.to_string(), "Parsing arguments `\"--opt-a\"` failed: None");
    }
    Ok(())
}
