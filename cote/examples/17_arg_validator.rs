use cote::prelude::*;
use cote::valid;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Cli {
    #[arg(valid = valid!(42))]
    foo: u64,

    #[arg(valid = valid!(["qux", "quux"]))]
    bar: Option<String>,

    #[pos(valid = valid!(4..42))]
    baz: Option<usize>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    assert!(Cli::parse(Args::from(["app", "--bar", "qux"])).is_err());

    assert!(Cli::parse(Args::from(["app", "--bar", "baz", "--foo=0"])).is_err());

    assert!(Cli::parse(Args::from(["app", "--bar", "baz", "68", "--foo=0"])).is_err());

    let cli = Cli::parse(Args::from(["app", "--bar", "qux", "--foo=42"]))?;

    assert_eq!(cli.foo, 42);
    assert_eq!(cli.bar.as_deref(), Some("qux"));

    let cli = Cli::parse(Args::from(["app", "--bar", "qux", "--foo=42", "6"]))?;

    assert_eq!(cli.foo, 42);
    assert_eq!(cli.bar.as_deref(), Some("qux"));
    assert_eq!(cli.baz, Some(6));

    Ok(())
}
