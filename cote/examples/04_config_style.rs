use cote::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(combine)]
pub struct Cli {
    #[arg(alias = "-d")]
    debug: bool,

    #[arg(alias = "-r")]
    recursive: bool,

    #[arg(alias = "-f")]
    force: bool,
}

fn main() -> Result<(), aopt::Error> {
    // set three options in one item
    let cli = Cli::parse(Args::from_array(["app", "-rdf"]))?;

    assert!(cli.debug);
    assert!(cli.recursive);
    assert!(cli.force);

    Ok(())
}
