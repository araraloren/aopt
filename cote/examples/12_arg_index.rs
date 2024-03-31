use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Cli {
    // `cmd` has a fixed position in default, you can't change it
    // and you can't both have a `cmd` and a `pos` at index 1
    #[cmd()]
    foo: bool,

    // `bar` has a index 2
    #[pos(index = "2", value = 42usize, hint = "[BAR]")]
    bar: Option<usize>,

    // option ignore the index value when matching with command line arguments
    #[arg(alias = "-b", help = "Set the string value of baz")]
    baz: String,

    // `quux` can accept position arguments at range from 3 to infinite
    #[pos(index = 3.., values = ["corge", "grault"])]
    quux: Vec<String>,
}
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let app = Cli::into_parser()?;

    assert_eq!(app["foo"].index(), Some(&Index::forward(1)));
    assert_eq!(app["bar"].index(), Some(&Index::forward(2)));
    assert_eq!(app["--baz"].index(), None);
    assert_eq!(app["quux"].index(), Some(&Index::range(Some(3), None)));

    Ok(())
}
