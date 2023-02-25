use aopt::{
    prelude::*,
    set::{SetCfg, SetOpt},
    Error,
};
use cote::{ParserIntoExtension, ParserExtractExtension};
use cote_derive::Cote;

#[derive(Cote, Debug, Clone)]
pub struct Copied<'a> {
    // What do you want
    #[arg(name = "-f")]
    from: &'a String,

    #[arg(name = "-t", value = ".", alias = "--to")]
    to: String,

    force: bool,

    #[arg(value = 42)]
    count: i64,
}

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let parser: AFwdParser = Copied::into_parser()?;

    Ok(())
}
