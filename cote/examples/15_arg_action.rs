use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Cli {
    // bool default has Action::Set
    #[arg(ty = bool, action = Action::Cnt)]
    foo: u64,

    // usize default has Action::App
    #[arg(action = Action::Set)]
    bar: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from_array([
        "app", "--foo", "--foo", "--bar=42", "--bar=88",
    ]))?;

    assert_eq!(cli.foo, 2);
    assert_eq!(cli.bar, 88);

    Ok(())
}
