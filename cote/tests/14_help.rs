use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote(help)]
pub struct Cli {
    #[allow(unused)]
    debug: bool,

    /// Query sub command
    #[allow(unused)]
    #[sub(foot = "Foot message of sub command query")]
    #[sub(name = "q", head = "Head message of sub command query")]
    query: Option<Query>,
}

#[derive(Debug, Cote)]
#[cote(help)]
pub struct Query {
    #[allow(unused)]
    #[arg(hint = "--row <usize>", help = "Set the row data of query")]
    row: usize,

    /// Set the format of query output
    #[allow(unused)]
    #[pos()]
    format: String,
}

#[test]
fn help() {
    assert!(help_impl().is_err());
}

fn help_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse(Args::from(["app", "q", "--help"]))?;
    // Output:
    // Usage: cli q [-h,--help] <--row <usize>> [ARGS]
    //
    // Head message of sub command query
    //
    // Options:
    // -h,--help          Display help message
    // --row <usize>      Set the row data of query
    //
    // Args:
    // format@1      Set the format of query output
    //
    // Foot message of sub command
    Ok(())
}
