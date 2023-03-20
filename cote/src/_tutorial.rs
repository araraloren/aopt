//!
//! # Documentation: Cote Tutorial
//!
//! 1. [Quick Start](#quick-start)
//! 2. [Configurating Struct](#configurating-struct)
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
//! ## Configurating Struct
//!
//! ### Configurating Policy
//!
//! Cote has three policy type built-in: [`fwd`](aopt::prelude::AFwdPolicy)、[`pre`](aopt::prelude::APrePolicy)
//! and [`delay`](aopt::prelude::ADelayPolicy).
//! If no `policy` configuration specific, [`fwd`](aopt::prelude::AFwdPolicy) will be using if no sub command.
//! Otherwise [`pre`](aopt::prelude::APrePolicy) will be used.
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(policy = delay)] // set policy to delay
//! pub struct Cli {
//!     debug: bool,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let GetoptRes { ret: _, mut parser } = Cli::parse_env_args()?;
//!     
//!     // `no_delay` only available in `ADelayPolicy`
//!     assert_eq!(parser.policy().no_delay().len(), 0);
//!     assert_eq!(Cli::try_extract(parser.optset_mut())?, Cli { debug: false });
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Configurating Help
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, // Generate help for current struct
//!     aborthelp, // Display help when error raised
//!     name = "app", // Set the usage name
//!     width = 50, // Set the maximum width of option help message
//!     usagew = 3, // Set the maximum count of item in usage
//!     head = "The head message display in help message",
//!     foot = "The foot message display in help message",
//! )]
//! pub struct Cli {
//!     /// Print debug message.
//!     debug: bool,
//!
//!     /// Set the name of client.
//!     name: String,
//!
//!     /// Switch to foo sub command.
//!     foo: Cmd,
//!
//!     /// Switch to bar sub command.
//!     bar: Cmd,
//!
//!     /// The second position argument.
//!     #[arg(index = "2")]
//!     arg: Pos<String>,
//!
//!     /// Collection of arguments start from position 3.
//!     #[arg(index = "3..")]
//!     args: Pos<Vec<String>>,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     // pass `--help` to program display help message
//!     Cli::parse(Args::from_array(["app", "--help"]))?;
//!     Ok(())
//! }
//! ```
//!
//! The help message output like this:
//!
//! ```!
//! Usage: app [-h,-?,--help] [--debug] <--name>
//!        <COMMAND> [ARGS]
//!
//! The head message display in help message
//!
//! Commands:
//!   foo@1      Switch to foo sub command.
//!   bar@1      Switch to bar sub command.
//!
//! Options:
//!   -h,-?,--help      Display help message
//!   --debug           Print debug message.
//!   --name            Set the name of client.
//!
//! Args:
//!   arg@2         The second position argument.
//!   args@3..      Collection of arguments start from position 3.
//!
//! The foot message display in help message
//! ```
//!
//! ### Configurating style
//!
//! - Add support for [`CombinedOption`](aopt::parser::UserStyle::CombinedOption).
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(combine)]
//! pub struct Cli {
//!     #[arg(alias = "-d")]
//!     debug: bool,
//!
//!     #[arg(alias = "-r")]
//!     recursive: bool,
//!
//!     #[arg(alias = "-f")]
//!     force: bool,
//! }
//!
//! fn main() -> Result<(), aopt::Error> {
//!     // set three options in one item
//!     let cli = Cli::parse(Args::from_array(["app", "-rdf"]))?;
//!
//!     assert!(cli.debug);
//!     assert!(cli.recursive);
//!     assert!(cli.force);
//!
//!     Ok(())
//! }
//! ```
//!
//! - Add support for [`EmbeddedValuePlus`](aopt::parser::UserStyle::EmbeddedValuePlus).
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(embedded)]
//! pub struct Cli {
//!     foo: String,
//! }
//!
//! fn main() -> Result<(), aopt::Error> {
//!     let cli = Cli::parse(Args::from_array(["app", "--foobar"]))?;
//!
//!     assert_eq!(cli.foo, "bar");
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration on field
//!
//! ### Add option
//!
//! ### Add sub command
//!
