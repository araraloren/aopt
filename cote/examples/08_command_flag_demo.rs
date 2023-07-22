use cote::*;

#[derive(Debug, Cote, PartialEq, Eq)]
pub struct Cli {
    #[cmd()]
    foo: bool, // Command flag has a fixed position 1,
    // and it's always force required
    #[pos(index = "2")]
    bar: Option<String>,
}

fn main() -> Result<(), aopt::Error> {
    let app = Cli::into_parser()?;

    assert_eq!(app["foo"].index(), Some(&Index::forward(1)));
    assert_eq!(app["bar"].index(), Some(&Index::forward(2)));

    let cli = Cli::parse(Args::from_array(["app", "foo", "42"]))?;

    assert_eq!(cli.bar.as_deref(), Some("42"));

    assert!(Cli::parse(Args::from_array(["app", "42", "foo"])).is_err());
    assert!(Cli::parse(Args::from_array(["app", "42"])).is_err());
    Ok(())
}
