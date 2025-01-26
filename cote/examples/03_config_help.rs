use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, // Generate help for current struct
     aborthelp, // Display help when error raised
     name = "app", // Set the usage name
     width = 50, // Set the maximum width of option help message
     usagew = 3, // Set the maximum count of item in usage
     head = "The head message display in help message",
     foot = "The foot message display in help message",
 )]
pub struct Cli {
    /// Print debug message.
    debug: bool,

    /// Set the name of client.
    name: String,

    /// Switch to foo sub command.
    foo: Cmd,

    /// Switch to bar sub command.
    bar: Cmd,

    /// The second position argument.
    #[pos(index = "2")]
    arg: String,

    /// Collection of arguments start from position 3.
    #[pos(index = 3..)]
    args: Vec<String>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // pass `--help` to program display help message
    assert!(Cli::parse(Args::from(["app", "--help"])).is_err());
    Ok(())
}
