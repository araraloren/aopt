use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote(
    help,
    head = "Set the head message here",
    foot = "Set the foot message here"
)]
pub struct Cli;

#[test]
fn head_foot() {
    assert!(head_foot_impl().is_ok());
}

fn head_foot_impl() -> Result<(), Box<dyn std::error::Error>> {
    Cli::parse(Args::from(["app", "-h"].into_iter()))?;
    // Output:
    //
    // Usage: cli [-h,--help]
    //
    // Set the head message here
    //
    // Options:
    //   -h,--help      Display help message
    //
    // Set the foot message here
    //
    Ok(())
}
