use cote::*;

#[derive(Debug, Cote, PartialEq, Eq)]
pub struct Cli {
    foo: Option<String>, // In default, it is generated into options.

    #[arg(name = "-b")]
    bar: Option<String>,
}

fn main() -> Result<(), aopt::Error> {
    let cli = Cli::parse(Args::from_array(["app"]))?;

    assert_eq!(cli.foo.as_deref(), None);
    assert_eq!(cli.bar.as_deref(), None);

    let cli = Cli::parse(Args::from_array(["app", "--foo", "bar", "-b=foo"]))?;

    assert_eq!(cli.foo.as_deref(), Some("bar"));
    assert_eq!(cli.bar.as_deref(), Some("foo"));

    let cli = Cli::parse(Args::from_array(["app", "-b", "foo", "--foo=bar"]))?;

    assert_eq!(cli.foo.as_deref(), Some("bar"));
    assert_eq!(cli.bar.as_deref(), Some("foo"));
    Ok(())
}
