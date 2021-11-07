use std::env::Args;

use getopt_rs::{prelude::{Result, SimpleParser}, set::{OptionSet, SimpleSet}, tools::{initialize_creator, initialize_prefix}, uid::UidGenerator};


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;



    Ok(())
}

fn parser_command_line(args: Args) -> Result<SimpleSet> {
    let mut set = SimpleSet::default();
    let mut parser = SimpleParser::<UidGenerator>::default();

    initialize_creator(&mut set);
    initialize_prefix(&mut set);

    // for [("-d=b", "--", "debug"), ("h=b", "--", "help"), "s=i", "--", ""]

    Ok(set)
}