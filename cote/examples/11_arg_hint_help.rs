use aopt::prelude::AOpt;
use cote::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Cli {
    /// Switch the mode to foo command
    #[cmd()]
    foo: bool,

    /// Set the value of bar
    #[pos(index = "2", value = 42usize, hint = "[BAR]")]
    bar: Option<usize>,

    #[arg(alias = "-b", help = "Set the string value of baz")]
    baz: String,

    #[pos(index = 3.., values = ["corge", "grault"])]
    quux: Vec<String>,
}

// Access the default value need invoke initialize handler, not recommend do this
fn default_value<T: ErasedTy>(opt: &mut AOpt) -> Result<Option<Vec<T>>, aopt::Error> {
    opt.accessor_mut().initializer_mut().values::<T>()
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut app = Cli::into_parser()?;

    assert_eq!(app["foo"].hint(), "foo@1");
    assert_eq!(app["bar"].hint(), "[BAR]");
    assert_eq!(app["--baz"].hint(), "-b, --baz");

    assert_eq!(app["foo"].help(), "Switch the mode to foo command");
    assert_eq!(app["bar"].help(), "Set the value of bar [42usize]");
    assert_eq!(app["--baz"].help(), "Set the string value of baz");

    assert_eq!(default_value::<String>(&mut app["--baz"])?, None);
    assert_eq!(default_value::<usize>(&mut app["bar"])?, Some(vec![42]));
    assert_eq!(
        default_value::<String>(&mut app["quux"])?,
        Some(vec!["corge".to_owned(), "grault".to_owned()])
    );

    // Currently only display default values are set in the attribute
    Cli::parse(Args::from(["app", "--help"]))?;

    Ok(())
}
