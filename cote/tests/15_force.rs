use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct Cli {
    #[allow(unused)]
    debug: bool,

    #[sub(force = false)]
    query: Option<Query>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Query {
    #[allow(unused)]
    row: usize,

    #[allow(unused)]
    #[pos()]
    format: String,
}

#[test]
fn force() {
    assert!(force_impl().is_ok());
}

fn force_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    assert_eq!(Cli::parse(Args::from(["app"].into_iter()))?.query, None);
    Ok(())
}
