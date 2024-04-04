use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    // `count` is an alias of `action = cote::Action::Cnt`
    #[arg(alias = "-v", ty = bool, count)]
    verbose: u64,
}

#[test]
fn action() {
    assert!(action_impl().is_ok());
}

fn action_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse(Args::from(["app", "-v", "-v", "-v"].into_iter()))?;
    assert_eq!(cli.verbose, 3);
    Ok(())
}
