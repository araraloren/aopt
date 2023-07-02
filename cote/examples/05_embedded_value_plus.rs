use cote::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(embedded)]
pub struct Cli {
    foo: String,
}

fn main() -> Result<(), aopt::Error> {
    let cli = Cli::parse(Args::from_array(["app", "--foobar"]))?;

    assert_eq!(cli.foo, "bar");

    Ok(())
}
