//! A flexible and typed getopt like command line tools for rust.
//!
//! ## Example
//!
//! ```ignore
//! use aopt::app::SingleApp;
//! use aopt::prelude::*;
//!
//! #[async_std::main]
//! async fn main() -> color_eyre::Result<()> {
//!     tracing_subscriber::fmt::fmt()
//!         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//!         .init();
//!     color_eyre::install()?;
//!     let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();
//!
//!     app.set_name("test-app".into());
//!     // add a new prefix `+` to app
//!     app.add_prefix("+".into());
//!     // add option `depth` with prefix `--` and type `i`
//!     app.add_opt("--depth=i")?
//!         .set_help("set the search depth of directory")
//!         .commit()?;
//!     // add option `source` with prefix `--` and type `a`
//!     app.add_opt("--source=a!")?
//!         .add_alias("+S")? // add an alias +S
//!         .set_help("add search source directory")
//!         .commit()?;
//!     // add option deactivate style `r` with prefix `-` and type `b`
//!     app.add_opt("-r=b/")?
//!         .set_help("disable recurse directory option")
//!         .commit()?;
//!     app.add_opt_cb(
//!         "--debug=b",
//!         simple_opt_cb!(|_, _, value| {
//!             if let Some(&v) = value.as_bool() {
//!                 if v {
//!                     println!("::: open debug mode");
//!                 }
//!             }
//!             Ok(Some(value))
//!         }),
//!     )?
//!     .commit()?;
//!
//!     app.run_async_mut(
//!         [
//!             "--depth=42",
//!             "+S",
//!             "./",
//!             "+Sfoo/",
//!             "--source",
//!             "bar/",
//!             "-/r",
//!         ]
//!         .into_iter(),
//!         |ret, app| async move {
//!             if ret {
//!                 println!("APP: {}", app.get_name());
//!                 for opt in app.opt_iter() {
//!                     let value = opt.get_value();
//!
//!                     if value.is_null() {
//!                         println!("OPTION: `{}` -> : not set", opt.get_name());
//!                     } else {
//!                         println!("OPTION: `{}` -> : {:?}", opt.get_name(), value);
//!                     }
//!                 }
//!             }
//!             Ok(())
//!         },
//!     )
//!     .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! The above code output:
//!
//! ```txt
//! APP: test-app
//! OPTION: `depth` -> : Int(42)
//! OPTION: `source` -> : Array(["./", "foo/", "bar/"])
//! OPTION: `r` -> : Bool(false)
//! OPTION: `debug` -> : not set
//! ```
//!
//! ## Setup
//!
//! Add following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! aopt = "0.5"
//! ```
//!
//! ### Enable `sync` feature
//!
//! If you want the utils of current crate implement [`Send`] and [`Sync`], you can enable `sync` feature.
//!
//! ```toml
//! [dependencies]
//! aopt = { version = "0.5", features = [ "sync" ] }
//! ```
//!
//! ## Feature
//!
//! In following example, the type of `parser` is `Parser<SimpleSet, DefaultService, ForwardPolicy>`.
//!
//! ### Option
//!
//! See the option create string help here: [`parse_option_str`](crate::opt::parse_option_str).
//!
//! The implementation of option is [`Opt`](crate::opt::Opt).
//! It is matched prefix and name of option argument.
//!
//! * Type support
//!
//! Common type options are built-in support.
//! Such as [`b`](crate::opt::opt::BoolOpt) and [`s`](crate::opt::opt::StrOpt).
//! With typed option support, you can keep typed value,
//! and customize the behavior when the user set it.
//! And you can add new type if necessary.
//!
//! #### Example
//!
//! ```ignore
//! parser.add_opt("--foo=b")?.commit()?; // add option `foo` with type `b`
//! parser.add_opt("--bar=s")?.commit()?; // add option `bar` with type `s`
//! ```
//!
//! #### Built-in type
//!
//! [`b`](crate::opt::opt::BoolOpt) with value type [`bool`].
//!
//! [`i`](crate::opt::opt::IntOpt) with value type [`i64`].
//!
//! [`u`](crate::opt::opt::UintOpt) with value type [`u64`].
//!
//! [`f`](crate::opt::opt::FltOpt) with value type [`f64`].
//!
//! [`s`](crate::opt::opt::StrOpt) with value type [`String`].
//!
//! [`a`](crate::opt::opt::ArrayOpt) with value type [`Vec`].
//!
//! #### Any type option
//!
//! [`PathOpt`](crate::opt::opt::PathOpt) is an exmaple option keep [`PathBuf`](std::path::PathBuf) inside.
//!
//! * Callback support
//!
//! The option can have associate [`OptCallback`](crate::opt::OptCallback).
//! The [`Parser`] will call it if user set the option.
//!
//! #### Example
//!
//! ```ignore
//! parser.add_opt_cb("--foo=s",
//!     simple_opt_cb!(|uid, set, value| {
//!     assert_eq!(value, OptValue::from("bar"));
//!     Ok(Some(value))
//! }))?.commit()?
//! // user can set the option `foo` like: `app.exe --foo=bar`
//! ```
//!
//! * Prefix support
//!
//! You can customize the prefix.
//!
//! #### Example
//!
//! ```ignore
//! parser.add_prefix("+".into()); // add support for prefix `+`
//! parser.add_opt("+F=a")?.commit()?;
//! // user can set the option `F` like: `app.exe +F foo +F bar`
//! ```
//!
//! * Alias support
//!
//! You can have one or more alias.
//!
//! #### Example
//!
//! ```ignore
//! parser.add_opt("--foo=s")?.add_alias("-f")?.commit()?;
//! // user can set the option `foo` like: `app.exe -f value`
//! ```
//!
//! * Value support
//!
//! You can keep a type value in the option, and it can have a default value.
//!
//! ### non-option
//!
//! In implementation side, [`NonOpt`](crate::opt::NonOpt) is based on [`Opt`](crate::opt::Opt).
//! Unlike the option matched with name and prefix of option argument.
//! The non-option is matched with [`OptIndex`](crate::opt::OptIndex)(based on 1) or name of non-option argument.
//!
//! * `p`: [`PosOpt`](crate::opt::nonopt::PosOpt)
//!
//! The `PosOpt` will match the index, and call the callback with type [`PosFn`](crate::opt::PosFn).
//!
//! #### Example
//!
//! ```ignore
//! parser.add_opt("--foo=b")?.commit()?; // add option `foo` with type `b`
//! parser.add_opt("--bar=s")?.commit()?; // add option `bar` with type `s`
//! parser.add_opt("arg=p@1",
//!     simple_pos_cb!(|uid, set, arg, index, value| {
//!     // will get `foo` inside value here
//!     assert_eq!(value, OptValue::from("foo"));
//!     Ok(Some(value))
//! }))?.commit()?;
//! // user can set the option like: `app.exe --foo --bar value foo`
//! ```
//!
//! * `c`: [`CmdOpt`](crate::opt::nonopt::CmdOpt)
//!
//! The `CmdOpt` is an specify [`PosOpt`](crate::opt::nonopt::PosOpt) with [`Forward`](crate::opt::OptIndex::Forward)(1).
//! It will match the name, and call the callback with type [`MainFn`](crate::opt::MainFn).
//!
//! #### Example
//!
//! ```ignore
//! parser.add_opt("--foo=b")?.commit()?; // add option `foo` with type `b`
//! parser.add_opt("--bar=s")?.commit()?; // add option `bar` with type `s`
//! parser.add_opt("show=c",
//!     simple_main_cb!(|uid, set, args, value| {
//!     assert_eq!(args, &["show", "foo"]);
//!     Ok(Some(value))
//! }))?.commit()?;
//! // user can set the option like: `app.exe show --foo --bar value foo`
//!
//! ```
//! * `m`: [`MainOpt`](crate::opt::nonopt::MainOpt)
//!
//! The `MainOpt` will always be called with the callback type [`MainFn`](crate::opt::MainFn).
//!
//! #### Example
//!
//! ```ignore
//! parser.add_opt("--foo=b")?.commit()?; // add option `foo` with type `b`
//! parser.add_opt("--bar=s")?.commit()?; // add option `bar` with type `s`
//! parser.add_opt("default_main=m",
//!     simple_main_cb!(|uid, set, args, value| {
//!     assert_eq!(args, &["foo", "bar"]);
//!     Ok(Some(value))
//! }))?.commit()?;
//! // user can set the option like: `app.exe --foo --bar value foo bar`
//! ```
//!
//! ### Policy
//!
//! [`Policy`] is responsible for analyze the input command argument,
//! and match it with option in given [`Set`].
//!
//! #### ForwardPolicy
//!
//! Parsing step of [`ForwardPolicy`](crate::parser::ForwardPolicy):
//!
//! * Go through the command line arguments.
//!
//!     If the argument like an option.
//!
//!     - Generate and process [`OptMatcher`](crate::proc::OptMatcher) of [`PSEqualWithValue`](crate::parser::ParserState::PSEqualWithValue)、[`PSArgument`](crate::parser::ParserState::PSArgument)、[`PSBoolean`](crate::parser::ParserState::PSBoolean)、[`PSMultipleOption`](crate::parser::ParserState::PSMultipleOption) and [`PSEmbeddedValue`](crate::parser::ParserState::PSEmbeddedValue). Invoke the callback if the any option matched.
//!
//!     - Return an Err if the option not matched and the strict flag is true.
//!
//!     Otherwise, add it to `NOA`(non-option argument) array.
//!
//! * Generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonCmd`](crate::parser::ParserState::PSNonCmd).
//!
//! * Go through the `NOA` array, and generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonPos`](crate::parser::ParserState::PSNonPos).
//!
//! * Generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonMain`](crate::parser::ParserState::PSNonMain).
//!
//! #### PrePolicy
//!
//! Parsing step of [`PrePolicy`](crate::parser::PrePolicy):
//!
//! * Go through the command line arguments.
//!
//!     If the argument like an option.
//!
//!     - Generate and process [`OptMatcher`](crate::proc::OptMatcher) of [`PSEqualWithValue`](crate::parser::ParserState::PSEqualWithValue)、[`PSArgument`](crate::parser::ParserState::PSArgument)、[`PSBoolean`](crate::parser::ParserState::PSBoolean)、[`PSMultipleOption`](crate::parser::ParserState::PSMultipleOption) and [`PSEmbeddedValue`](crate::parser::ParserState::PSEmbeddedValue), call the callback if the any option matched.
//!
//!     - __Add it to `NOA` array if the option not matched.__
//!
//!     Otherwise, add it to `NOA`(non-option argument) array.
//!
//! * Generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonCmd`](crate::parser::ParserState::PSNonCmd).
//!
//! * Go through the `NOA` array, and generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonPos`](crate::parser::ParserState::PSNonPos).
//!
//! * Generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonMain`](crate::parser::ParserState::PSNonMain).
//!
//! #### DelayPolicy
//!
//! Parsing step of [`DelayPolicy`](crate::parser::DelayPolicy):
//!
//! * Go through the command line arguments.
//!
//!     If the argument like an option.
//!
//!     - Generate and process [`OptMatcher`](crate::proc::OptMatcher) of [`PSDelayEqualWithValue`](crate::parser::ParserState::PSDelayEqualWithValue)、[`PSDelayArgument`](crate::parser::ParserState::PSDelayArgument)、[`PSDelayBoolean`](crate::parser::ParserState::PSDelayBoolean)、[`PSDelayMultipleOption`](crate::parser::ParserState::PSDelayMultipleOption) and [`PSDelayEmbeddedValue`](crate::parser::ParserState::PSDelayEmbeddedValue). __Add the callback invoke context to [`ValueKeeper`](crate::parser::ValueKeeper) array if any option matched__.
//!
//!     - Return an Err if the option not matched and the strict flag is true.
//!
//!     Otherwise, add it to `NOA`(non-option argument) array.
//!
//! * Generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonCmd`](crate::parser::ParserState::PSNonCmd).
//!
//! * Go through the `NOA` array, and generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonPos`](crate::parser::ParserState::PSNonPos).
//!
//! * __Invoke the callback that saved in [`ValueKeeper`](crate::parser::ValueKeeper) array.__
//!
//! * Generate and process [`NonOptMatcher`](crate::proc::NonOptMatcher) of [`PSNonMain`](crate::parser::ParserState::PSNonMain).
pub mod app;
pub mod arg;
pub mod ctx;
pub mod err;
pub mod opt;
pub mod parser;
pub mod proc;
pub mod set;
pub mod uid;

#[macro_use]
extern crate tracing;

use crate::arg::ArgStream;
use crate::parser::DynParser;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::Service;
use crate::set::Set;

pub use crate::app::SingleApp;
pub use crate::err::Error;
pub use crate::err::Result;

/// Create a [`Ustr`](ustr::Ustr) from `&str`.
pub fn gstr(s: &str) -> ustr::Ustr {
    ustr::Ustr::from(s)
}

pub fn getopt_dynparser<I, ITER, S, SS>(
    iter: ITER,
    parsers: Vec<&mut DynParser<S, SS>>,
) -> Result<Option<&mut DynParser<S, SS>>>
where
    I: Into<String>,
    ITER: Iterator<Item = I>,
    S: Set,
    SS: Service<S>,
{
    let args: Vec<String> = iter.map(|v| v.into()).collect();
    let count = parsers.len();
    let mut index = 0;

    for parser in parsers {
        let mut stream = ArgStream::from(args.clone().into_iter());

        match parser.parse(&mut stream) {
            Ok(rv) => {
                if rv {
                    return Ok(Some(parser));
                }
            }
            Err(e) => {
                if e.is_special() && index + 1 != count {
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
        index += 1;
    }
    Ok(None)
}

pub fn getopt_parser<I, ITER, S, SS, P>(
    iter: ITER,
    parsers: Vec<&mut Parser<S, SS, P>>,
) -> Result<Option<&mut Parser<S, SS, P>>>
where
    I: Into<String>,
    ITER: Iterator<Item = I>,
    S: Set,
    SS: Service<S>,
    P: Policy<S, SS>,
{
    let args: Vec<String> = iter.map(|v| v.into()).collect();
    let count = parsers.len();
    let mut index = 0;

    for parser in parsers {
        let mut stream = ArgStream::from(args.clone().into_iter());

        match parser.parse(&mut stream) {
            Ok(rv) => {
                if rv {
                    return Ok(Some(parser));
                }
            }
            Err(e) => {
                if e.is_special() && index + 1 != count {
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
        index += 1;
    }
    Ok(None)
}

/// Parse the given string sequence, return the first matched [`DynParser`]: `getopt!($Iterator, $($DynParser),+)`.
///
/// # Returns
///
/// Will return an Ok(Some([`DynParser`])) if any [`DynParser`] parsing successed, otherwise return `Ok(None)`.  
///
/// # Example
///
/// ```rust
/// use aopt::err::Result;
/// use aopt::prelude::*;
///
/// fn main() -> Result<()> {
///     let mut parser = DynParser::<SimpleSet, DefaultService>::new_policy(ForwardPolicy::default());
///     let mut pre_parser = DynParser::<SimpleSet, DefaultService>::new_policy(PrePolicy::default());
///
///     parser.add_opt("-a=b!")?.commit()?;
///     parser.add_opt("--bopt=i")?.commit()?;
///     parser
///         .add_opt_cb("c=p@-1",
///         simple_pos_cb!(|_, _, arg, _, value| {
///             assert_eq!(arg, "foo");
///             Ok(Some(value))
///         }))?
///         .commit()?;
///
///     pre_parser.add_opt("-d=a")?.commit()?;
///     pre_parser.add_opt("--eopt=s")?.commit()?;
///     {
///         let ret = getoptd!(
///             &mut ["-a", "--bopt=42", "foo"].iter().map(|&v| String::from(v)),
///             parser,
///             pre_parser
///         )?;
///
///         assert!(ret.is_some());
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("-a")?,
///             Some(&OptValue::from(true))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("--bopt")?,
///             Some(&OptValue::from(42i64))
///         );
///     }
///     {
///         let ret = getoptd!(
///             &mut ["-dbar", "-d", "foo", "--eopt=pre", "foo"]
///                 .iter()
///                 .map(|&v| String::from(v)),
///             parser,
///             pre_parser
///         )?;
///         assert!(ret.is_some());
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("-d")?,
///             Some(&OptValue::from(vec!["bar".to_owned(), "foo".to_owned()]))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("--eopt")?,
///             Some(&OptValue::from("pre"))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().get_service().get_noa().as_slice(),
///             ["foo"]
///         );
///     }
///     {
///         assert!(getoptd!(
///             &mut ["-dbar", "-d", "foo", "--eopt=pre", "foo"]
///                 .iter()
///                 .map(|&v| String::from(v)),
///             parser
///         )
///         .is_err());
///     }
///     Ok(())
/// }
///```
pub use aopt_macro::getoptd;

/// Parse the given string sequence, return the first matched [`Parser`]: `getopt!($Iterator, $($Parser),+)`.
///
/// # Returns
///
/// Will return an Ok(Some([`Parser`])) if any [`Parser`] parsing successed, otherwise return `Ok(None)`.
///
/// # Example
///
/// ```rust
/// use aopt::err::Result;
/// use aopt::prelude::*;
///
/// fn main() -> Result<()> {
///     {
///         let mut parser = Parser::<SimpleSet, DefaultService, ForwardPolicy>::default();
///
///         parser.add_opt("-a=b!")?.commit()?;
///         parser.add_opt("--bopt=i")?.commit()?;
///         parser
///             .add_opt_cb(
///                 "c=p@-1",
///                 simple_pos_cb!(|_, _, arg, _, value| {
///                     assert_eq!(arg, "foo");
///                     Ok(Some(value))
///                 }),
///             )?
///             .commit()?;
///
///         let ret = getopt!(
///             &mut ["-a", "--bopt=42", "foo"].iter().map(|&v| String::from(v)),
///             parser,
///         )?;
///
///         assert!(ret.is_some());
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("-a")?,
///             Some(&OptValue::from(true))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("--bopt")?,
///             Some(&OptValue::from(42i64))
///         );
///     }
///     {
///         let mut pre_parser = Parser::<SimpleSet, DefaultService, PrePolicy>::default();
///
///         pre_parser.add_opt("-d=a")?.commit()?;
///         pre_parser.add_opt("--eopt=s")?.commit()?;
///
///         let ret = getopt!(
///             &mut ["-dbar", "-d", "foo", "--eopt=pre", "foo"]
///                 .iter()
///                 .map(|&v| String::from(v)),
///             pre_parser
///         )?;
///         assert!(ret.is_some());
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("-d")?,
///             Some(&OptValue::from(vec!["bar".to_owned(), "foo".to_owned()]))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().get_set().get_value("--eopt")?,
///             Some(&OptValue::from("pre"))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().get_service().get_noa().as_slice(),
///             ["foo"]
///         );
///     }
///     Ok(())
/// }
///```
pub use aopt_macro::getopt;

/// Generate help message of Set: `getopt_help!($Set, $($cmd_name),*)`.
///
/// # Example
/// ```ignore
/// use aopt::err::Result;
/// use aopt::prelude::*;
/// use aopt_help::prelude::*;
///
/// fn main() -> Result<()> {
///     let mut parser = ForwardParser::default();
///
///     getopt_add!(parser, "--aopt=s!", "Help message of aopt")?;
///     getopt_add!(
///         parser,
///         "--bopt=b",
///         alias = "-b",
///         help = "Help message of bopt"
///     )?;
///     getopt_add!(parser, "--copt=a", name = "选项c", prefix = "-")?;
///     getopt_add!(
///         parser,
///         "--dopt=b/",
///         "Help message of dopt",
///         simple_opt_cb!(|_, _, v| {
///             Ok(Some(v))
///         })
///     )?;
///     getopt_add!(
///         parser,
///         "--eopt=i",
///         callback = simple_opt_cb!(|_, _, v| {
///             Ok(Some(v))
///         })
///     )?;
///     getopt_add!(
///         parser,
///         "fopt=p",
///         index = OptIndex::forward(1),
///         callback = simple_pos_cb!(|_, _, _, _, v| {
///             Ok(Some(v))
///         })
///     )?;
///     getopt_add!(parser, "--gopt=u", default = OptValue::from(42u64))?;
///
///     getopt_help!(parser.get_set()).print_cmd_help(None).unwrap();
///
///     Ok(())
/// }
/// ```
///
/// Above code will generate output like:
/// ```txt
/// usage: simple-exmaple <--aopt> [-b,--bopt] [-选项c] [--/dopt] [--eopt] [--gopt] **ARGS**
///
/// Simple example for aopt-help.
///
/// POS:
///   fopt@1
///
/// OPT:
///   --aopt         s      Help message of aopt
///   -b,--bopt      b      Help message of bopt
///   -选项c         a
///   --/dopt        b      Help message of dopt
///   --eopt         i
///   --gopt         u
///
/// Create by araraloren v0.5.42
/// ```
pub use aopt_macro::getopt_help;

/// Add option to `Parser`: `getopt_add!($Parser, $Create-String, $Help?, $Callback?, $($key=$value),*)`.
///
/// `$Create-String` is option create string,
/// `$Help` and `$Callback` are optional, and the available key list are:
///
/// - [`help`](crate::set::CreateInfo::set_help)
///
/// - [`name`](crate::set::CreateInfo::set_name)
///
/// - [`prefix`](crate::set::CreateInfo::set_prefix)
///
/// - [`index`](crate::set::CreateInfo::set_index)
///
/// - [`default`](crate::set::CreateInfo::set_default_value)
///
/// - [`hint`](crate::set::CreateInfo::set_hint)
///
/// - [`alias`](crate::set::CreateInfo::add_alias)
///
/// - [`callback`](crate::parser::Parser::add_callback)
///
/// # Example
/// ```rust
/// use aopt::err::Result;
/// use aopt::prelude::*;
///
/// fn main() -> Result<()> {
///     let mut parser = ForwardParser::default();
///
///     getopt_add!(parser, "--aopt=s!", "Help message of aopt")?;
///     getopt_add!(
///         parser,
///         "--bopt=b",
///         alias = "-b",
///         help = "Help message of bopt"
///     )?;
///     getopt_add!(parser, "--copt=a", name = "选项c", prefix = "-")?;
///     getopt_add!(
///         parser,
///         "--dopt=b/",
///         "Help message of dopt",
///         simple_opt_cb!(|_, _, v| {
///             assert_eq!(v, OptValue::from(false));
///             Ok(Some(v))
///         })
///     )?;
///     getopt_add!(
///         parser,
///         "--eopt=i",
///         callback = simple_opt_cb!(|_, _, v| {
///             assert_eq!(v, OptValue::from(42i64));
///             Ok(Some(v))
///         })
///     )?;
///     getopt_add!(
///         parser,
///         "fopt=p",
///         index = OptIndex::forward(1),
///         callback = simple_pos_cb!(|_, _, arg, _, v| {
///             assert_eq!(arg, "foo");
///             Ok(Some(v))
///         })
///     )?;
///     getopt_add!(parser, "--gopt=u", default = OptValue::from(42u64))?;
///     getopt_add!(
///         parser,
///         "hopt=m",
///         callback = simple_main_cb!(|_, set: &SimpleSet, _, v| {
///             assert_eq!(set["--aopt"].get_help(), "Help message of aopt");
///             assert_eq!(set["--bopt"].get_help(), "Help message of bopt");
///             assert_eq!(set["--dopt"].get_help(), "Help message of dopt");
///             assert_eq!(set["选项c"].get_name(), "选项c");
///             assert_eq!(set["选项c"].get_prefix(), "-");
///             assert_eq!(set["gopt"].get_value(), &OptValue::from(42u64));
///             assert_eq!(set["aopt"].get_value(), &OptValue::from("bar"));
///             assert_eq!(set["bopt"].get_value(), &OptValue::from(true));
///             Ok(Some(v))
///         })
///     )?;
///     getopt!(
///         &mut ["--aopt", "bar", "-b", "foo", "--/dopt", "--eopt=42"]
///             .iter()
///             .map(|&v| String::from(v)),
///         parser,
///     )?;
///
///     Ok(())
/// }
/// ```
pub use aopt_macro::getopt_add;

pub mod prelude {
    pub use crate::arg::ArgStream;
    pub use crate::ctx::Context;
    pub use crate::ctx::NonOptContext;
    pub use crate::ctx::OptContext;
    pub use crate::getopt;
    pub use crate::getopt_add;
    pub use crate::getopt_dynparser;
    pub use crate::getopt_help;
    pub use crate::getopt_parser;
    pub use crate::getoptd;
    pub use crate::gstr;
    pub use crate::opt::Alias;
    pub use crate::opt::ArrayCreator;
    pub use crate::opt::BoolCreator;
    pub use crate::opt::Callback;
    pub use crate::opt::CmdCreator;
    pub use crate::opt::FltCreator;
    pub use crate::opt::Help;
    pub use crate::opt::HelpInfo;
    pub use crate::opt::Identifier;
    pub use crate::opt::Index;
    pub use crate::opt::IntCreator;
    pub use crate::opt::MainCreator;
    pub use crate::opt::Name;
    pub use crate::opt::NonOpt;
    pub use crate::opt::Opt;
    pub use crate::opt::OptCallback;
    pub use crate::opt::OptIndex;
    pub use crate::opt::OptValue;
    pub use crate::opt::Optional;
    pub use crate::opt::PosCreator;
    pub use crate::opt::SimpleMainFn;
    pub use crate::opt::SimpleMainFnMut;
    pub use crate::opt::SimpleOptFn;
    pub use crate::opt::SimpleOptFnMut;
    pub use crate::opt::SimplePosFn;
    pub use crate::opt::SimplePosFnMut;
    pub use crate::opt::StrCreator;
    pub use crate::opt::Type;
    pub use crate::opt::UintCreator;
    pub use crate::opt::Value;
    pub use crate::parser::DefaultService;
    pub use crate::parser::DelayParser;
    pub use crate::parser::DelayPolicy;
    pub use crate::parser::DynParser;
    pub use crate::parser::ForwardParser;
    pub use crate::parser::ForwardPolicy;
    pub use crate::parser::Parser;
    pub use crate::parser::Policy;
    pub use crate::parser::PreParser;
    pub use crate::parser::PrePolicy;
    pub use crate::parser::Service;
    pub use crate::parser::SimpleService;
    pub use crate::proc::Info;
    pub use crate::proc::Matcher;
    pub use crate::proc::NonOptMatcher;
    pub use crate::proc::OptMatcher;
    pub use crate::proc::Proc;
    pub use crate::set::CreateInfo;
    pub use crate::set::CreatorSet;
    pub use crate::set::OptionSet;
    pub use crate::set::PrefixSet;
    pub use crate::set::Set;
    pub use crate::set::SetIndex;
    pub use crate::set::SimpleSet;
    pub use crate::simple_main_cb;
    pub use crate::simple_main_mut_cb;
    pub use crate::simple_opt_cb;
    pub use crate::simple_opt_mut_cb;
    pub use crate::simple_pos_cb;
    pub use crate::simple_pos_mut_cb;
    pub use crate::uid::{Uid, UidGenerator};
    pub use ustr::Ustr;
    pub use ustr::UstrMap;
}
