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
//! 3. [Configurating Field](#configurating-field)
//!     1. [Options](#options)
//!     2. [Positionals](#positionals)
//!     3. [Command Flags](#command-flags)
//!     4. [Sub Commands](#sub-commands)
//! 4. [Configurating Options, Command flags and Positionals](#configurating-options-command-flags-and-positionals)
//!     1. [Configurating the name and alias](#configurating-the-name-and-alias)
//!     2. [Configurating the hint, help and default value](#configurating-the-hint-help-and-default-value)
//!     3. [Configurating the index](#configurating-the-index)
//!     4. [Make the option force required](#make-the-option-force-required)
//!     5. [Configurating action](#configurating-action)
//!     6. [Configurating handler](#configurating-handler)
//!     7. [Validate values](#validate-values)
//!     8. [Add "no delay" option](#add-no-delay-option)
//! 5. [Configurating Sub Commands](#configurating-sub-commands)
//!     1. [Configurating Policy](#configurating-policy)
//!     2. [Configurating name and alias](#configurating-name-and-alias)
//!     3. [Configurating help message](#configurating-help-message)
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
//!   se@1      Search the given directory
//!   ls@1      List the given directory
//!
//! Options:
//!   -h,-?,--help           Display help message
//!   -d,--debug             Print debug message
//!   -c,--config [CFG]      Set the configuration path ["default.json"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! Error:
//!    0: Parsing command `se` failed: InnerCtx { uid: 1, name: Some(--depth), style: Style::Argument, arg: Some("www"), index: 1, total: 3 }
//!    1: Can not find option `--depth`
//!    2: Can not convert value `www` to usize
//!    3: invalid digit found in string
//!
//! Location:
//!    src\main.rs:82
//!
//! Backtrace omitted.
//! Run with RUST_BACKTRACE=1 environment variable to display it.
//! Run with RUST_BACKTRACE=full to include source snippets.
//! ```
//!
//! ## Configurating Struct
//!
//! ### Configurating Policy
//!
//! Cote has three policy types built-in: [`fwd`](aopt::prelude::AFwdPolicy)、[`pre`](aopt::prelude::APrePolicy)
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
//! Specify `help` in `cote` attribute will automate generate help message for current application.
//! And `aborthelp` will automate display the help message if any error raised.
//! 
//! The default name of the application is the name of the current package, use `name` custom it,
//! i.e., the result of `String::from(env!("CARGO_PKG_NAME"))`.
//! 
//! The default maximum length of the option help message is 40, use `width` custom it.
//! The default maximum count of usage option item is 10, use `usagew` custom it.
//! 
//! The text set by `head` will display after usage, in default it is description of package,
//! i.e., the result of `String::from(env!("CARGO_PKG_DESCRIPTION"))`.
//! 
//! The text set by `foot` will display at the bottom, in default it is result of
//! `format!("Create by {} v{}", env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_VERSION"))`.
//! 
//! #### Example
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
//! The option styles support by default are:
//! 
//! - [`EqualWithValue`](aopt::parser::UserStyle::EqualWithValue)
//! 
//! Options such as `--opt=value`, the value of option is set after `=`.
//! 
//! - [`Argument`](aopt::parser::UserStyle::Argument)
//! 
//! Options such as `--opt value`, the value of option is next argument.
//! 
//! - [`EmbeddedValue`](aopt::parser::UserStyle::EmbeddedValue)
//! 
//! Options such as `-o42`, the value `42` of option is embedded in string. 
//! The style only support one letter option.
//! 
//! - [`Boolean`](aopt::parser::UserStyle::Boolean)
//! 
//! Options such as `--opt`, in general, it is named flag, the value type of option is always `bool`.
//!
//! - Add support for [`CombinedOption`](aopt::parser::UserStyle::CombinedOption).
//! 
//! Options such as `-abcd`, thus set both boolean options `-a`, `-b`, `-c` and `-d`.
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
//! Options such as `--opt42`, the value `42` of option is embedded in string.
//! The style only supports options which name lengths bigger than 2.
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
//! ## Configurating Field
//!
//! ### Options
//!
//! In default or specific the attribute `arg`, the fields of struct are generated into options.
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
//! ### Command Flags
//!
//! Specific the attribute `cmd` will let you create a sub command flag.
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! pub struct Cli {
//!     #[cmd()]
//!     foo: bool, // Command flag has a fixed position 1,
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
//! ### Sub Commands
//!
//! Specific the attribute `sub` will let you create a sub commands.
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Cli {
//!     #[arg()]
//!     bar: usize,
//!
//!     #[sub(alias = "z")]
//!     baz: Option<Baz>,
//!
//!     #[sub(alias = "x")]
//!     qux: Option<Qux>,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Baz {
//!     grault: bool,
//!
//!     waldo: Option<String>,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Qux {
//!     garply: bool,
//!
//!     fred: String,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let cli = Cli::parse(Args::from_array(["app", "--bar=42", "z"]))?;
//!
//!     assert_eq!(cli.bar, 42);
//!     assert_eq!(
//!         cli.baz,
//!         Some(Baz {
//!             grault: false,
//!             waldo: None
//!         })
//!     );
//!     assert_eq!(cli.qux, None);
//!
//!     let cli = Cli::parse(Args::from_array([
//!         "app", "--bar=42", "x", "--fred", "plugh",
//!     ]))?;
//!
//!     assert_eq!(cli.bar, 42);
//!     assert_eq!(cli.baz, None);
//!     assert_eq!(
//!         cli.qux,
//!         Some(Qux {
//!             garply: false,
//!             fred: "plugh".to_owned()
//!         })
//!     );
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Configurating Options, Command flags and Positionals
//!
//! ### Configurating the name and alias
//! 
//! The default name of positionals and command flags is the name of the field.
//! 
//! The default name of options consists of prefixs and identifiers of the field. 
//! The default prefix is `--` if count of characters bigger than 1, otherwise `-` is using.
//! You can use `name` or `alias` configure the name and alias of the option.
//! For prefix information reference [`PrefixOptValidator`](aopt::prelude::PrefixOptValidator).
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
//! ### Configurating the hint, help and default value
//! 
//! Hint is displayed on usage or the left side of item information. 
//! In default, hint message is generated from the name and alias of item,
//! use `hint` custom the hint information of item.
//! Help is displayed on the right side of item information.
//! Use `help` configure the help information of item.
//! The default values will be display in help message if it is set.
//! 
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
//! Running the code, it's output should be:
//! 
//! ```!
//! Usage: cli [-h,-?,--help] <-b,--baz> <COMMAND> [ARGS] 
//!
//! Generate help message for command line program
//! 
//! Commands:
//!   foo@1      Switch the mode to foo command
//! 
//! Options:
//!   -h,-?,--help      Display help message
//!   -b,--baz          Set the string value of baz
//! 
//! Args:
//!   [BAR]         Set the value of bar [42usize]
//!   quux@3..
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//!
//! ### Configurating the index
//!
//! Index is only support positions and command flags.
//! For command flags, the index is fixed position `@1` by default.
//! For more informations about index, reference [`Index`](aopt::prelude::Index).
//!
//! #### Example1
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
//! #### Example2
//! 
//! For the item configured by `pos`, the index is automating generated start form 1 
//! if no index set.
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
//! ### Make the option force required
//! 
//! In default, options, positionals and command flags is force required.
//! Wrap the type with `Option` can make the item optional.
//! Using `force` you can configure the positionals and options force required.
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
//!     #[arg(force = true)]
//!     qux: Option<i64>,
//!
//!     // Using `force` you can force set `--quux` to optional in `arg`.
//!     // But the parse will raise error when extract `Cli` from `CoteApp`
//!     // if the option has no default value
//!     #[arg(force = false, values = ["need"])]
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
//!
//! ### Configurating action
//!
//! The type that implements [`Infer`](aopt::prelude::Infer) has different [`Action`](aopt::prelude::Action).
//! The [`Action`](aopt::prelude::Action) defines the behavior when saving the value.
//! For more information, see [`Action::process`](aopt::prelude::Action#method.process) and [`AOpt`](aopt::prelude::AOpt).
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help)]
//! pub struct Cli {
//!     // bool default has Action::Set
//!     #[arg(ty = bool, action = Action::Cnt)]
//!     foo: u64,
//!
//!     // usize default has Action::App
//!     #[arg(action = Action::Set)]
//!     bar: usize,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let cli = Cli::parse(Args::from_array(["app", "--foo", "--foo", "--bar=42", "--bar=88"]))?;
//!
//!     assert_eq!(cli.foo, 2);
//!     assert_eq!(cli.bar, 88);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Configurating handler
//!
//! Using `on`, `fallback` attribute configure the handler which will be called when
//! option set.
//! Using `then` attribute configure the store behavior when saving value.
//!
//! - `on`
//!
//!     + `cote`
//!
//!     Will be invoked if struct parsed successfully.
//!     Because the name of [`Main`](aopt::opt::Main) option will be generate automate.
//!     So you can't get the return value currently.
//!
//!     + `arg` or `pos`
//!
//!     Will be invoked if option set by user.
//!     The return value will be saved as value of option.
//!
//!     + `sub`
//!
//!     Not support, set the handler on struct type using `cote`.
//!
//! - `fallback`
//!     
//! Same as `on` except if the handler returns `Ok(None)`, the default handler will be invoked.
//!
//! - `then`
//!
//! Using with `on` and `fallback`, do nothing without `on` and `fallback`.
//! It will responded for saving the raw value and value.
//!
//! ```rust
//! use std::{fmt::Debug, ops::Deref};
//! use cote::prelude::*;
//!
//! // The handler must be a generic function.
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, on = display_cli::<P>)]
//! pub struct Cli {
//!     #[arg(on = empty_handler::<P>, then = foo_storer::<P>)]
//!     foo: u64,
//!
//!     #[sub(force = false)]
//!     bar: Option<Bar>,
//!
//!     #[sub(force = false)]
//!     qux: Option<Qux>,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help)]
//! pub struct Bar {
//!     #[arg(force = false, fallback = debug_of_bar::<P>)]
//!     debug: bool,
//!
//!     #[pos()]
//!     quux: String,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, fallback = process_qux::<P>, then = unreachable_storer::<P>)]
//! pub struct Qux {
//!     #[cmd(name = "c")]
//!     corge: bool,
//!
//!     grault: Option<i64>,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     //! unwrap the failure of return value
//!     Cli::parse_env_args()?.ret.unwrap();
//!
//!     Ok(())
//! }
//!
//! fn display_cli<P>(set: &mut P::Set, _: &mut P::Ser) -> Result<Option<()>, aopt::Error>
//! where
//!     P: Policy,
//!     P::Set: SetValueFindExt + Set,
//! {
//!     println!("Got client: {:?}", Cli::try_extract(set)?);
//!     Ok(None)
//! }
//!
//! fn empty_handler<P>(
//!     _: &mut P::Set,
//!     _: &mut P::Ser,
//!     value: Option<ctx::Value<u64>>,
//! ) -> Result<Option<u64>, aopt::Error>
//! where
//!     P: Policy,
//! {
//!     Ok(value.map(|mut v| v.take()))
//! }
//!
//! fn foo_storer<P>(
//!     uid: Uid,
//!     set: &mut P::Set,
//!     _: &mut P::Ser,
//!     raw: Option<&RawVal>,
//!     val: Option<u64>,
//! ) -> Result<bool, aopt::Error>
//! where
//!     P: Policy,
//!     P::Set: SetValueFindExt + Set,
//! {
//!     let has_value = val.is_some();
//!
//!     // Set the value if return Some(Value)
//!     if let Some(val) = val {
//!         if let Some(opt) = set.get_mut(uid) {
//!             let (raw_handler, handler) = opt.accessor_mut().handlers();
//!
//!             if let Some(raw_value) = raw {
//!                 raw_handler.push(raw_value.clone());
//!             }
//!             println!("Saving the value of `--foo` to {}", val + 1);
//!             // modify the value, plus one
//!             handler.push(val + 1);
//!         }
//!     }
//!
//!     Ok(has_value)
//! }
//!
//! fn debug_of_bar<P>(
//!     _: &mut P::Set,
//!     _: &mut P::Ser,
//!     raw: ctx::RawVal,
//!     value: ctx::Value<bool>,
//! ) -> Result<Option<()>, aopt::Error>
//! where
//!     P: Policy,
//! {
//!     println!(
//!         "Got value of `--debug`: {:?} --> {}",
//!         raw.deref(),
//!         value.deref()
//!     );
//!     // if return None, the parser will call default handler of current option
//!     Ok(None)
//! }
//!
//! fn process_qux<P>(_: &mut P::Set, _: &mut P::Ser) -> Result<Option<()>, aopt::Error>
//! where
//!     P: Policy,
//!     P::Set: SetValueFindExt + Set,
//! {
//!     println!("return Ok(None) call the default handler of Qux");
//!     Ok(None)
//! }
//!
//! fn unreachable_storer<P>(
//!     _: Uid,
//!     _: &mut P::Set,
//!     _: &mut P::Ser,
//!     _: Option<&RawVal>,
//!     _: Option<()>,
//! ) -> Result<bool, aopt::Error>
//! where
//!     P: Policy,
//!     P::Set: SetValueFindExt + Set,
//! {
//!     unreachable!("Never go here")
//! }
//! ```
//!
//! - Output of command line `cli --foo 6`:
//!
//! ```!
//! Saving the value of `--foo` to 7
//! Got client: Cli { foo: 7, bar: None, qux: None }
//! ```
//!
//! - Output of command line `cli --foo 8 bar a2i`:
//!
//! ```!
//! Saving the value of `--foo` to 9
//! Got client: Cli { foo: 9, bar: Some(Bar { debug: false, quux: "a2i" }), qux: None }
//! ```
//!
//! - Output of command line `cli --foo 8 bar a2i --debug`:
//!
//! ```!
//! Saving the value of `--foo` to 9
//! Got value of `--debug`: RawVal("true") --> true
//! Got client: Cli { foo: 9, bar: Some(Bar { debug: false, quux: "a2i" }), qux: None }
//! ```
//!
//! - Output of command line `cli --foo 9 qux c`:
//!
//! ```!
//! Saving the value of `--foo` to 10
//! return Ok(None) call the default handler of Qux
//! Got client: Cli { foo: 9, bar: None, qux: Some(Qux { corge: true, grault: None }) }
//! ```
//!
//! - Output of command line `cli --foo 9 qux c --grault=42`:
//!
//! ```!
//! Saving the value of `--foo` to 10
//! return Ok(None) call the default handler of Qux
//! Got client: Cli { foo: 9, bar: None, qux: Some(Qux { corge: true, grault: Some(42) }) }
//! ```
//!
//! ### Validate values
//!
//! You can using `valid` check the value inside attribute.
//! Using [`valid!`](crate::valid!) generate struct implemented [`Validate`](crate::valid::Validate)
//! for the valid attribute.
//!
//! ```rust
//! use cote::prelude::*;
//! use cote::valid;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help)]
//! pub struct Cli {
//!     #[arg(valid = valid!(42))]
//!     foo: u64,
//!
//!     #[arg(valid = valid!(["qux", "quux"]))]
//!     bar: Option<String>,
//!
//!     #[pos(valid = valid!(4..42))]
//!     baz: Option<usize>,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     assert!(Cli::parse(Args::from_array(["app", "--bar", "qux"])).is_err());
//!
//!     assert!(Cli::parse(Args::from_array(["app", "--bar", "baz", "--foo=0"])).is_err());
//!
//!     assert!(Cli::parse(Args::from_array(["app", "--bar", "baz", "68", "--foo=0"])).is_err());
//!
//!     let cli = Cli::parse(Args::from_array(["app", "--bar", "qux", "--foo=42"]))?;
//!
//!     assert_eq!(cli.foo, 42);
//!     assert_eq!(cli.bar.as_deref(), Some("qux"));
//!
//!     let cli = Cli::parse(Args::from_array(["app", "--bar", "qux", "--foo=42", "6"]))?;
//!
//!     assert_eq!(cli.foo, 42);
//!     assert_eq!(cli.bar.as_deref(), Some("qux"));
//!     assert_eq!(cli.baz, Some(6));
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Add "no delay" option
//!
//! When using [`DelayPolicy`](aopt::prelude::DelayPolicy), the option process(invoke handler)
//! after `Cmd` and `Pos` style.
//! Sometimes we need the option process like [`FwdPolicy`](aopt::prelude::FwdPolicy) does,
//! that is process before `Cmd` and `Pos`.
//!
//!```rust
//! use cote::prelude::*;
//! use std::ops::Deref;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(policy = delay, help)]
//! pub struct Cli {
//!     #[cmd(on = cmd_order::<P>)]
//!     foo: bool,
//!
//!     #[arg(on = assert_order::<P>)]
//!     bar: usize,
//!
//!     #[pos(on = assert_order::<P>, index = 2)]
//!     baz: usize,
//!
//!     #[arg(on = assert_order::<P>, nodelay)]
//!     qux: usize,
//! }
//!
//! fn cmd_order<P: Policy>(_: &mut P::Set,  ser: &mut P::Ser) -> Result<Option<bool>, aopt::Error>
//! where
//!     P::Ser: ServicesValExt,
//! {
//!     let order = ser.sve_val_mut::<usize>()?;
//!     *order += 1;
//!     let order = *order;
//!     assert_eq!(order, 2);
//!     println!("Order {}", order);
//!     Ok(Some(true))
//! }
//!
//! fn assert_order<P: Policy>(
//!     _: &mut P::Set,
//!     ser: &mut P::Ser,
//!     mut val: ctx::Value<usize>,
//! ) -> Result<Option<usize>, aopt::Error>
//! where
//!     P::Ser: ServicesValExt,
//! {
//!     let order = ser.sve_val_mut::<usize>()?;
//!     *order += 1;
//!     let order = *order;
//!     assert_eq!(order, *val.deref());
//!     println!("Order {}", order);
//!     Ok(Some(val.take()))
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!     let mut app = Cli::into_app()?;
//!
//!     app.set_app_data(0usize)?;
//!     app.run_mut_with(
//!         ["app", "foo", "--bar=4", "--qux=1", "3"].into_iter(),
//!         |_, app| {
//!             let cli = Cli::try_extract(app.optset_mut())?;
//!             assert_eq!(cli.foo, true);
//!             assert_eq!(cli.bar, 4);
//!             assert_eq!(cli.qux, 1);
//!             assert_eq!(cli.baz, 3);
//!             Ok(())
//!         },
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Configurating Sub Commands
//!
//! Using `sub` attribute define sub command.
//!
//! ```rust
//! use cote::prelude::*;
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Cli {
//!     #[arg(alias = "-g")]
//!     age: usize,
//!
//!     /// Help message of eat sub command
//!     #[sub()]
//!     eat: Option<Eat>,
//!
//!     /// Help message of sport sub command
//!     #[sub(policy = pre)]
//!     sport: Option<Sport>,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Eat {
//!     
//!     /// Which meal did you have?
//!     #[arg(alias = "-m")]
//!     meal: String,
//!
//!     /// What did you wat?
//!     #[pos(value = "rice")]
//!     what: Option<String>,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Sport {
//!     /// Go for a walk.
//!     #[sub()]
//!     walk: Option<Walk>,
//!
//!     /// Play some games.
//!     #[sub()]
//!     play: Option<Play>,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Walk {
//!     #[arg(name = "-d", value = 3usize)]
//!     distance: usize,
//! }
//!
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Play {
//!     /// Which game do you want to play?
//!     #[pos(value = "Mario")]
//!     game: String,
//! }
//!
//! fn main() -> color_eyre::Result<()> {
//!     color_eyre::install()?;
//!
//!     let cli = Cli::parse_env()?;
//!
//!     println!("You age is set to {}", cli.age);
//!     if let Some(eat) = cli.eat {
//!         println!("You {} are going to eat {}", eat.meal, eat.what.unwrap());
//!     } else if let Some(sport) = cli.sport {
//!         if let Some(walk) = sport.walk {
//!             println!("You are going to walk {} kilometers", walk.distance);
//!         } else if let Some(play) = sport.play {
//!             println!("You are going to play game {}", play.game);
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Configurating Policy
//!
//! The default [`Policy`](aopt::prelude::Policy) of sub command is [`FwdPolicy`](aopt::prelude::FwdPolicy).
//! For the sub commands to have sub commands, you should use [`PrePolicy`](aopt::prelude::PrePolicy) instead.
//! For example, `sport` sub command does have two sub commands, it is configured with `#[sub(policy = pre)]`.
//! Without `policy = pre`, you will got output when running `cli -g=42 sport walk -d 4`:
//!
//! ```!
//! Usage: cli sport [-h,-?,--help] <COMMAND>
//! Generate help message for command line program
//!
//! Commands:
//!   walk@1      Go for a walk.
//!   play@1      Play some games.
//!
//! Options:
//!   -h,-?,--help      Display help message
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! Error:
//!    0: Parsing command `sport` failed: None
//!    1: Can not find option `-d`
//!
//! Location:
//!    src\main.rs:90
//!
//! Backtrace omitted.
//! Run with RUST_BACKTRACE=1 environment variable to display it.
//! Run with RUST_BACKTRACE=full to include source snippets.
//! ```
//! And the right output should be:
//! ```!
//! You age is set to 42
//! You are going to walk 4 kilometers
//! ```
//!
//! ### Configurating name and alias
//! 
//! Using `name` and `alias` you can configure the name and alias of sub commands in `sub` attribute.
//! The name and alias will affect how to set the sub command and help message of sub command.
//! With follow change:
//! 
//! ```no_run
//! #[derive(Debug, Cote, PartialEq, Eq)]
//! #[cote(help, aborthelp)]
//! pub struct Cli {
//!     #[arg(alias = "-g")]
//!     age: usize,
//!
//!     /// Help message of eat sub command
//!     #[sub(alias = "e")]
//!     eat: Option<Eat>,
//!
//!     /// Help message of sport sub command
//!     #[sub(name = "sp", policy = pre)]
//!     sport: Option<Sport>,
//! }
//! ```
//! 
//! The output of commands `cli -g22 e --help` is:
//! 
//! ```!
//! Usage: cli e [-h,-?,--help] <-m,--meal> [ARGS]
//!
//! Generate help message for command line program
//!
//! Options:
//!   -h,-?,--help      Display help message
//!   -m,--meal         Which meal did you have?
//!
//! Args:
//!   what@1      What did you wat? ["rice"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//! 
//! ### Configurating help message
//! 
//! Using `hint`, `help`, `head`, `foot` you can configure the help message of sub commands.
//! 
