//!
//! # Documentation: Cote Tutorial
//!
//! 1. [Quick Start](#quick-start)
//!     1. [Help message generate](#help-message-generate)
//!     2. [Running](#running)
//! 2. [Configurating Struct](#configurating-struct)
//!     1. [Configurating Policy](#configurating-policy)
//!     2. [Configurating Help](#configurating-help)
//!     3. [Configurating User Style](#configurating-user-style)
//!
//!
//! ## Quick Start
//!
//! Using [`Cote`](crate::cote_derive::Cote) derive you can quick setup a application.
//!
//! ```no_run
//! use std::path::PathBuf;
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
//!     #[pos(value = ".", help = "Set the clean directory")]
//!     dest: Option<PathBuf>,
//! }
//!
//! #[derive(Debug, Cote)]
//! #[cote(help)]
//! pub struct List {
//!     /// Enable recursive mode
//!     recursive: bool,
//!     #[pos(value = ".", help = "Set the clean directory")]
//!     dest: Option<PathBuf>,
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
//!  se@1       Search the given directory
//!  ls@1       List the given directory
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
//!   dest@1      Set the list directory ["."]
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
//!   se@1       Search the given directory
//!   ls@1       List the given directory
//!
//! Options:
//!   -h,-?,--help           Display help message
//!   -d,--debug             Print debug message
//!   -c,--config [CFG]      Set the configuration path ["default.json"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! Error:
//!    0: Failed at command `se` with `["--depth", "www"]`: Can not find option `--depth`:
//! Can not convert value `www` to usize: ParseIntError { kind: InvalidDigit },
//! inner_ctx = InnerCtx { uid: 1, name: Some(--depth), style: Style::Argument, arg: Some("www"), index: 1, total: 3 }
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
//!     assert_eq!(parser.policy().no_delay().map(|v|v.len()), Some(0));
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
//!     #[pos(index = "2")]
//!     arg: String,
//!
//!     /// Collection of arguments start from position 3.
//!     #[pos(index = "3..")]
//!     args: Vec<String>,
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
//! ### Configurating User Style
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
//! ### Options
//!
//! Specific the attribute `arg`, the fields of struct are generated into options.
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! pub struct Cli {
//!     foo: Option<String>, // In default, it is generated into options.
//!
//!     #[arg(name = "-b")]
//!     bar: Option<String>,
//! }
//!
//! fn main() -> Result<(), aopt::Error> {
//!     let cli = Cli::parse(Args::from_array(["app"]))?;
//!
//!     assert_eq!(cli.foo.as_deref(), None);
//!     assert_eq!(cli.bar.as_deref(), None);
//!
//!     let cli = Cli::parse(Args::from_array(["app", "--foo", "bar", "-b=foo"]))?;
//!
//!     assert_eq!(cli.foo.as_deref(), Some("bar"));
//!     assert_eq!(cli.bar.as_deref(), Some("foo"));
//!
//!     let cli = Cli::parse(Args::from_array(["app", "-b", "foo", "--foo=bar", ]))?;
//!
//!     assert_eq!(cli.foo.as_deref(), Some("bar"));
//!     assert_eq!(cli.bar.as_deref(), Some("foo"));
//!     Ok(())
//! }
//! ```
//!
//! ### Positionals
//!
//! Specific the attribute `pos` if you want to match the command line arguments by position.
//!
//! ```
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! pub struct Cli {
//!     #[pos()]
//!     foo: Option<String>, // if not specific, index will automate generated base on field index
//!
//!     #[pos(index = "2")]
//!     bar: Option<String>,
//! }
//!
//! fn main() -> Result<(), aopt::Error> {
//!     let app = Cli::into_app()?;
//!
//!     assert_eq!(app["foo"].index(), Some(&Index::forward(1)));
//!     assert_eq!(app["bar"].index(), Some(&Index::forward(2)));
//!
//!     let cli = Cli::parse(Args::from_array(["app"]))?;
//!
//!     assert_eq!(cli.foo.as_deref(), None);
//!     assert_eq!(cli.bar.as_deref(), None);
//!
//!     let cli = Cli::parse(Args::from_array(["app", "42", "foo"]))?;
//!
//!     assert_eq!(cli.foo.as_deref(), Some("42"));
//!     assert_eq!(cli.bar.as_deref(), Some("foo"));
//!     Ok(())
//! }
//! ```
//!
//! ### Commands
//!
//! Specific the attribute `cmd` will let you create a sub command flag.
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! pub struct Cli {
//!     #[cmd()]
//!     foo: bool, // Command has a fixed position 1,
//!                // and it's always force required
//!
//!     #[pos(index = "2")]
//!     bar: Option<String>,
//! }
//!
//! fn main() -> Result<(), aopt::Error> {
//!     let app = Cli::into_app()?;
//!
//!     assert_eq!(app["foo"].index(), Some(&Index::forward(1)));
//!     assert_eq!(app["bar"].index(), Some(&Index::forward(2)));
//!
//!     let cli = Cli::parse(Args::from_array(["app", "foo", "42"]))?;
//!
//!     assert_eq!(cli.bar.as_deref(), Some("42"));
//!
//!     assert!(Cli::parse(Args::from_array(["app", "42", "foo"])).is_err());
//!     assert!(Cli::parse(Args::from_array(["app", "42"])).is_err());
//!     Ok(())
//! }
//! ```
//!
//! ### Configurations of attribute `arg`, `pos` and `cmd`
//!
//! #### Configure the name and alias
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! pub struct Cli {
//!     #[cmd(name = "foo", alias = "f")]
//!     cmd: bool,
//!
//!     // set the name of position, for access the option from index operator
//!     #[pos(name = "bar", index = "2")]
//!     pos: usize,
//!
//!     // set the option name with prefix
//!     #[arg(name = "--baz", alias = "-b")]
//!     opt: String,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let app = Cli::into_app()?;
//!
//!     assert_eq!(app["foo"].name(), "foo");
//!     assert_eq!(app["bar"].name(), "bar");
//!     assert_eq!(app["--baz"].name(), "--baz");
//!     assert_eq!(app["-b"].name(), "--baz");
//!
//!     let cli = Cli::parse(Args::from_array(["app", "--baz", "qux", "foo", "42"]))?;
//!
//!     assert_eq!(cli.cmd, true);
//!     assert_eq!(cli.pos, 42);
//!     assert_eq!(cli.opt, "qux");
//!
//!     let cli = Cli::parse(Args::from_array(["app", "f", "-b=quux", "88"]))?;
//!
//!     assert_eq!(cli.cmd, true);
//!     assert_eq!(cli.pos, 88);
//!     assert_eq!(cli.opt, "quux");
//!
//!     Ok(())
//! }
//! ```
//!
//! #### Configure the hint, help and default value
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help)]
//! pub struct Cli {
//!     /// Switch the mode to foo command
//!     #[cmd()]
//!     foo: bool,
//!
//!     /// Set the value of bar
//!     #[pos(index = "2", value = 42usize, hint = "[BAR]")]
//!     bar: Option<usize>,
//!
//!     #[arg(alias = "-b", help = "Set the string value of baz")]
//!     baz: String,
//!
//!     #[pos(index = "3..", values = ["corge", "grault"])]
//!     quux: Vec<String>,
//! }
//!
//! // Access the default value need invoke initialize handler, not recommend do this
//! fn default_value<T: ErasedTy>(opt: &mut AOpt) -> Result<Option<Vec<T>>, aopt::Error> {
//!     opt.accessor_mut().initializer_mut().values::<T>()
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let mut app = Cli::into_app()?;
//!
//!     assert_eq!(app["foo"].hint(), "foo@1");
//!     assert_eq!(app["bar"].hint(), "[BAR]");
//!     assert_eq!(app["--baz"].hint(), "-b,--baz");
//!
//!     assert_eq!(app["foo"].help(), "Switch the mode to foo command");
//!     assert_eq!(app["bar"].help(), "Set the value of bar [42usize]");
//!     assert_eq!(app["--baz"].help(), "Set the string value of baz");
//!
//!     assert_eq!(default_value::<String>(&mut app["--baz"])?, None);
//!     assert_eq!(default_value::<usize>(&mut app["bar"])?, Some(vec![42]));
//!     assert_eq!(
//!         default_value::<String>(&mut app["quux"])?,
//!         Some(vec!["corge".to_owned(), "grault".to_owned()])
//!     );
//!
//!     // Currently only display default values are set in the attribute
//!     Cli::parse(Args::from_array(["app", "--help"]))?;
//!
//!     Ok(())
//! }
//! ```
//!
//! #### Configure the index
//!
//! For more informations about index, reference [`Index`](aopt::prelude::Index).
//!
//! ##### Example1
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help)]
//! pub struct Cli {
//!     // `cmd` has a fixed position in default, you can't change it
//!     // and you can't both have a `cmd` and a `pos` at index 1
//!     #[cmd()]
//!     foo: bool,
//!
//!     // `bar` has a index 2
//!     #[pos(index = "2", value = 42usize, hint = "[BAR]")]
//!     bar: Option<usize>,
//!
//!     // option ignore the index value when matching with command line arguments
//!     #[arg(alias = "-b", help = "Set the string value of baz")]
//!     baz: String,
//!
//!     // `quux` can accept position arguments at range from 3 to infinite
//!     #[pos(index = "3..", values = ["corge", "grault"])]
//!     quux: Vec<String>,
//! }
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let app = Cli::into_app()?;
//!
//!     assert_eq!(app["foo"].index(), Some(&Index::forward(1)));
//!     assert_eq!(app["bar"].index(), Some(&Index::forward(2)));
//!     assert_eq!(app["--baz"].index(), None);
//!     assert_eq!(app["quux"].index(), Some(&Index::range(Some(3), None)));
//!
//!     Ok(())
//! }
//! ```
//!
//! ##### Example2
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help)]
//! pub struct Cli {
//!     // `bar` has an index 1, it is automated generate by derive macro
//!     #[pos(value = 42usize)]
//!     bar: Option<usize>,
//!
//!     // option ignore the index value when matching with command line arguments
//!     #[arg(alias = "-b", help = "Set the string value of baz")]
//!     baz: String,
//!
//!     // `quux` can accept position arguments at range 3 or 4
//!     #[pos(index = "3..5")]
//!     quux: Vec<String>,
//! }
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let app = Cli::into_app()?;
//!
//!     assert_eq!(app["bar"].index(), Some(&Index::forward(1)));
//!     assert_eq!(app["--baz"].index(), None);
//!     assert_eq!(app["quux"].index(), Some(&Index::range(Some(3), Some(5))));
//!
//!     let app = Cli::parse(Args::from_array([
//!         "app", // index 0
//!         "88", // index 1
//!         "--baz", // option --baz
//!         "foo", // value of option --baz
//!         "ignore", // index 2
//!         "what", // index 3
//!         "where", // index 4
//!     ]))?;
//!
//!     assert_eq!(app.bar, Some(88));
//!     assert_eq!(app.baz, "foo");
//!     assert_eq!(app.quux, vec!["what".to_owned(), "where".to_owned()]);
//!
//!     Ok(())
//! }
//! ```
//!
//! #### Make the option force required
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help)]
//! pub struct Cli {
//!     // `cmd` is force required in default, you can't change it
//!     #[cmd()]
//!     foo: bool,
//!
//!     // `Option` make the `pos` optional in default
//!     #[pos(index = "2", value = 42usize)]
//!     bar: Option<usize>,
//!
//!     // Without `Option`, `--baz` is force required
//!     #[arg(alias = "-b", help = "Set the string value of baz")]
//!     baz: String,
//!
//!     // Using `force` you can force set the option to force required
//!     #[arg(force)]
//!     qux: Option<i64>,
//!
//!     // Using `noforce` you can force set `--quux` to optional in `arg`.
//!     // But the parse will raise error when extract `Cli` from `CoteApp`
//!     // if the option has no default value
//!     #[arg(noforce, values = ["need"])]
//!     quux: Vec<String>,
//! }
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     assert!(Cli::parse(Args::from_array(["app", "--baz=6"])).is_err());
//!
//!     assert!(Cli::parse(Args::from_array(["app", "foo", "--baz=6"])).is_err());
//!
//!     assert!(Cli::parse(Args::from_array(["app", "--qux", "-5", "foo", "--baz=6"])).is_ok());
//!
//!     Ok(())
//! }
//! ```
