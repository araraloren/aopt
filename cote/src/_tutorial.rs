//!
//! # Documentation: Cote Tutorial
//!
//! 1. [Quick Start](#qucik-start)
//!
//!
//! ## Quick Start
//!
//! Using [`Cote`](crate::cote_derive::Cote) derive you can quick setup a application.
//!
//! ```rust
//! use std::path::PathBuf;
//! use aopt::opt::Pos;
//! use aopt::Error;
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote)]
//! #[cote(
//!     name = "cli", // set the name of usage
//!     help, // generate help and display help when `--help` set
//!     aborthelp, // display help if any error raised
//!     width = 50
//! )]
//! pub struct Cli {
//!     /// Print debug message
//!     #[arg(alias = "-d")]
//!     debug: bool,
//!
//!     /// Set the configuration path
//!     #[arg(alias = "-c", value = "default.json", hint = "-c,--config [CFG]")]
//!     config: Option<PathBuf>,
//!
//!     /// Search the given directory
//!     #[sub(name = "se")]
//!     search: Option<Search>,
//!
//!     /// List the given directory
//!     #[sub(name = "ls", head = "List the given directory")]
//!     list: Option<List>,
//! }
//!
//! #[derive(Debug, Cote)]
//! #[cote(help)]
//! pub struct Search {
//!     /// Set the depth of search
//!     depth: usize, // without `Option` mean force required
//!
//!     #[arg(index = "1", value = ".", help = "Set the clean directory")]
//!     dest: Option<Pos<PathBuf>>,
//! }
//!
//! #[derive(Debug, Cote)]
//! #[cote(help)]
//! pub struct List {
//!     /// Enable recursive mode
//!     recursive: bool,
//!     #[arg(index = "1", value = ".", help = "Set the clean directory")]
//!     dest: Option<Pos<PathBuf>>,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let cli = Cli::parse_env()?;
//!
//!     if cli.debug {
//!         println!("enable debug mode, will print debug message")
//!     }
//!     if let Some(cfg) = cli.config.as_deref() {
//!         println!("loading config from {:?}", cfg);
//!     }
//!     if let Some(list) = cli.list.as_ref() {
//!         println!(
//!             "list the directory `{:?}` with recursive({})",
//!             list.dest.as_deref(),
//!             list.recursive
//!         );
//!     } else if let Some(search) = cli.search.as_ref() {
//!         println!(
//!             "search the file under directory `{:?}` with depth {}",
//!             search.dest.as_deref(),
//!             search.depth
//!         );
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Help message generate
//!
//! - Output of `cli --help`:
//!
//! ```!
//! Usage: cli [-h,-?,--help] [-d,--debug] [-c,--config [CFG]] <COMMAND>
//!
//! Generate help message for command line program
//!
//! Commands:
//!  se       Search the given directory
//!  ls       List the given directory
//!
//! Options:
//!   -h,-?,--help           Display help message
//!   -d,--debug             Print debug message
//!   -c,--config [CFG]      Set the configuration path ["default.json"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//!
//! - Output of `cli ls --help`:
//!
//! ```!
//! Usage: cli ls [-h,-?,--help] [--recursive] [ARGS]
//!
//! List the given directory
//!
//! Options:
//!   -h,-?,--help      Display help message
//!   --recursive       Enable recursive mode
//!
//! Args:
//!   dest      Set the list directory ["."]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//!
//! ### Running
//!
//! Output of `cli se --depth 2`:
//!
//! ```!
//! loading config from "default.json"
//! search the file under directory `Some(".")` with depth 2
//! ```
//!
//! #### `aborthelp`
//!
//! If code generate with cote configuration `aborthelp`.
//! When the option match failed, program will first
//! print help message, then display the error message.
//!
//! Output of `cli se --depth www` or `cli se --depth`:
//!
//! ```!
//! Usage: cli [-h,-?,--help] [-d,--debug] [-c,--config [CFG]] <COMMAND>
//!
//! Generate help message for command line program
//!
//! Commands:
//!   se       Search the given directory
//!   ls       List the given directory
//!
//! Options:
//!   -h,-?,--help           Display help message
//!   -d,--debug             Print debug message
//!   -c,--config [CFG]      Set the configuration path ["default.json"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! Error:
//!    0: failed at sub command `Search` with args `Args { inner: [RawVal("cli"), RawVal("--depth")] }: OptionNotFound("--depth")
//!
//! Location:
//!    src\main.rs:56
//!
//! Backtrace omitted.
//! Run with RUST_BACKTRACE=1 environment variable to display it.
//! Run with RUST_BACKTRACE=full to include source snippets.
//! error: process didn't exit successfully: `cli se --depth` (exit code: 1)
//! ```
//!
//! ## Configuration on struct
//!
//! ## Configuration on field
//!
//! ### Add option
//!
//! ### Add sub command
//!
