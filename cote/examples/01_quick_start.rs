use cote::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Cote)]
#[cote(
     name = "cli", // set the name of usage
     help, // generate help and display help when `--help` set
     aborthelp, // display help if any error raised
     width = 50
 )]
pub struct Cli {
    /// Print debug message
    #[arg(alias = "-d")]
    debug: bool,

    /// Set the configuration path
    #[arg(alias = "-c", value = "default.json", hint = "-c,--config [CFG]")]
    config: Option<PathBuf>,

    /// Search the given directory
    #[sub(name = "se")]
    search: Option<Search>,

    /// List the given directory
    #[sub(name = "ls", head = "List the given directory")]
    list: Option<List>,
}

#[derive(Debug, Cote)]
#[cote(help)]
pub struct Search {
    /// Set the depth of search
    depth: usize, // without `Option` mean force required

    #[pos(value = ".", help = "Set the clean directory")]
    dest: Option<PathBuf>,
}

#[derive(Debug, Cote)]
#[cote(help)]
pub struct List {
    /// Enable recursive mode
    recursive: bool,
    #[pos(value = ".", help = "Set the clean directory")]
    dest: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse_env()?;

    if cli.debug {
        println!("enable debug mode, will print debug message")
    }
    if let Some(cfg) = cli.config.as_deref() {
        println!("loading config from {:?}", cfg);
    }
    if let Some(list) = cli.list.as_ref() {
        println!(
            "list the directory `{:?}` with recursive({})",
            list.dest.as_deref(),
            list.recursive
        );
    } else if let Some(search) = cli.search.as_ref() {
        println!(
            "search the file under directory `{:?}` with depth {}",
            search.dest.as_deref(),
            search.depth
        );
    }
    Ok(())
}
