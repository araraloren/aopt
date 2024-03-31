use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Cli {
    // `bar` has an index 1, it is automated generate by derive macro
    #[pos(value = 42usize)]
    bar: Option<usize>,

    // option ignore the index value when matching with command line arguments
    #[arg(alias = "-b", help = "Set the string value of baz")]
    baz: String,

    // `quux` can accept position arguments at range 3 or 4
    #[pos(index = 3..5)]
    quux: Vec<String>,
}
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let parser = Cli::into_parser()?;

    assert_eq!(parser["bar"].index(), Some(&Index::forward(1)));
    assert_eq!(parser["--baz"].index(), None);
    assert_eq!(
        parser["quux"].index(),
        Some(&Index::range(Some(3), Some(5)))
    );

    let app = Cli::parse(Args::from([
        "app",    // index 0
        "88",     // index 1
        "--baz",  // option --baz
        "foo",    // value of option --baz
        "ignore", // index 2
        "what",   // index 3
        "where",  // index 4
    ]))?;

    assert_eq!(app.bar, Some(88));
    assert_eq!(app.baz, "foo");
    assert_eq!(app.quux, vec!["what".to_owned(), "where".to_owned()]);

    Ok(())
}
