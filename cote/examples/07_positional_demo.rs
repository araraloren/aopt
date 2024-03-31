use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
pub struct Cli {
    #[pos()]
    foo: Option<String>, // if not specific, index will automate generated base on field index

    #[pos(index = "2")]
    bar: Option<String>,
}

fn main() -> Result<(), aopt::Error> {
    let app = Cli::into_parser()?;

    assert_eq!(app["foo"].index(), Some(&Index::forward(1)));
    assert_eq!(app["bar"].index(), Some(&Index::forward(2)));

    let cli = Cli::parse(Args::from(["app"]))?;

    assert_eq!(cli.foo.as_deref(), None);
    assert_eq!(cli.bar.as_deref(), None);

    let cli = Cli::parse(Args::from(["app", "42", "foo"]))?;

    assert_eq!(cli.foo.as_deref(), Some("42"));
    assert_eq!(cli.bar.as_deref(), Some("foo"));
    Ok(())
}
