use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(policy = delay)] // set policy to delay
pub struct Cli {
    debug: bool,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let GetoptRes { ret: _, mut parser } = Cli::parse_env_args()?;

    assert_eq!(parser.policy().no_delay().map(|v| v.len()), Some(0));
    assert_eq!(Cli::try_extract(parser.optset_mut())?, Cli { debug: false });

    Ok(())
}
