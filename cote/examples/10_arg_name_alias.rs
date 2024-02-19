use cote::*;

#[derive(Debug, Cote, PartialEq, Eq)]
pub struct Cli {
    #[cmd(name = "foo", alias = "f")]
    cmd: bool,

    // set the name of position, for access the option from index operator
    #[pos(name = "bar", index = "2")]
    pos: usize,

    // set the option name with prefix
    #[arg(name = "--baz", alias = "-b")]
    opt: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let app = Cli::into_parser()?;

    assert_eq!(app["foo"].name(), "foo");
    assert_eq!(app["bar"].name(), "bar");
    assert_eq!(app["--baz"].name(), "--baz");
    assert_eq!(app["-b"].name(), "--baz");

    let cli = Cli::parse(Args::from(["app", "--baz", "qux", "foo", "42"]))?;

    assert_eq!(cli.cmd, true);
    assert_eq!(cli.pos, 42);
    assert_eq!(cli.opt, "qux");

    let cli = Cli::parse(Args::from(["app", "f", "-b=quux", "88"]))?;

    assert_eq!(cli.cmd, true);
    assert_eq!(cli.pos, 88);
    assert_eq!(cli.opt, "quux");

    Ok(())
}
