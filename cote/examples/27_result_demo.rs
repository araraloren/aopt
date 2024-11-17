use cote::prelude::*;

#[derive(Debug, Cote)]
pub struct Cli {
    foo: Result<String, cote::Error>, // In default, it is generated into options.

    #[arg(name = "-b")]
    bar: Result<String, cote::Error>,
}

fn main() -> Result<(), aopt::Error> {
    let cli = Cli::parse(Args::from(["app"]))?;

    assert_eq!(cli.foo.as_deref().ok(), None);
    assert_eq!(cli.bar.as_deref().ok(), None);

    let cli = Cli::parse(Args::from(["app", "--foo", "bar", "-b=foo"]))?;

    assert_eq!(cli.foo.as_deref().ok(), Some("bar"));
    assert_eq!(cli.bar.as_deref().ok(), Some("foo"));

    let cli = Cli::parse(Args::from(["app", "-b", "foo", "--foo=bar"]))?;

    assert_eq!(cli.foo.as_deref().ok(), Some("bar"));
    assert_eq!(cli.bar.as_deref().ok(), Some("foo"));
    Ok(())
}
