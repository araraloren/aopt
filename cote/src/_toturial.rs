//!
//! # Documentation: Cote Toturial
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
//! #[cote(policy = pre, name = "cli", help, width = 50)]
//! pub struct Cli {
//!     /// Print debug message
//!     debug: bool,
//! 
//!     /// Set the configuration path
//!     #[arg(value = "default.json", hint = "--config [CFG]")]
//!     config: Option<PathBuf>,
//! 
//!     /// Search the given directory
//!     #[sub(name = "s")]
//!     search: Option<Search>,
//! }
//! 
//! #[derive(Debug, Cote)]
//! #[cote(help)]
//! pub struct Search {
//!     /// Set the depth of search
//!     depth: usize,  // without `Option` mean force required
//! 
//!     #[arg(index = "1", value = ".", help = "Set the clean directory")]
//!     dest: Option<Pos<PathBuf>>,
//! }
//! 
//! fn main() -> Result<(), Error> {
//!     let cli = Cli::parse_env()?;
//! 
//!     if cli.debug {
//!         println!("enable debug mode, will print debug message")
//!     }
//!     if let Some(cfg) = cli.config.as_deref() {
//!         println!("loading config from {:?}", cfg);
//!     }
//!     if let Some(search) = cli.search.as_ref() {
//!         println!("search the file with depth {}", search.depth);
//!         println!("search the file under {:?}", search.dest.as_deref());
//!     }
//!     Ok(())
//! }
//! ```