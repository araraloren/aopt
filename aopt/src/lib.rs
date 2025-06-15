#![doc = include_str!("../README.md")]
pub mod ctx;
pub mod guess;
pub mod opt;
pub mod parser;
pub mod set;
#[cfg(feature = "shell")]
pub mod shell;
pub mod value;

pub use crate::acore::args;
pub use crate::acore::err;
pub use crate::acore::error;
pub use crate::acore::failure;
pub use crate::acore::map;
pub use crate::acore::str;
pub use crate::acore::trace;
pub use crate::acore::ARef;
pub use crate::acore::HashMap;
pub use crate::acore::Uid;

pub(crate) use aopt_core as acore;
pub(crate) use aopt_shell as ashell;

pub use crate::err::Error;
pub use crate::err::Result;

/// Get the [`TypeId`](std::any::TypeId) of type `T`.
pub(crate) fn typeid<T: ?Sized + 'static>() -> std::any::TypeId {
    std::any::TypeId::of::<T>()
}

#[derive(Debug)]
pub struct GetoptRes<R, T> {
    pub ret: R,

    pub parser: T,
}

/// Parse the string sequence with given [`Parser`](crate::parser::Parser).
///
/// # Returns
///
/// For style `getopt!(..., &mut parser1, &mut parser2)`,
/// will return an Ok([`GetoptRes`]\(T is the type of matched [`Parser`](crate::parser::Parser)\)) if any [`Parser`](crate::parser::Parser) parsing successed.
/// For style `getopt!(..., "first" => &mut parser1, "second" => &mut parser2)`,
/// will return an Ok([`GetoptRes`]\(T is the literal type\)) if any [`Parser`](crate::parser::Parser) parsing successed.
///
/// Will return Err([`Error::no_parser_matched()`]) if all [`Parser`](crate::parser::Parser) parsing failed, otherwise return Err(_).
/// # Example
///
/// ```rust
/// # use aopt::err::Result;
/// # use aopt::prelude::*;
/// #
/// # fn main() -> Result<()> {
/// let mut parser = AFwdParser::default();
/// let mut pre_parser = AFwdParser::default();
///
/// pre_parser.set_prepolicy(true);
/// {
///     parser.add_opt("-a=b!")?;
///     parser.add_opt("--bopt=i")?;
///     parser.add_opt("c=p@-0")?.on(
///         |_, ctx: &mut Ctx| {
///             let val = ctx.value::<String>()?;
///             let args = ctx.args();
///             assert_eq!(args[0], OsStr::new("foo"));
///             Ok(Some(val))
///         },
///     )?;
///
///     assert!(getopt!(Args::from(["-a", "--bopt=42", "foo"]), &mut parser).is_ok());
///     assert_eq!(parser.find_val::<bool>("-a")?, &true);
///     assert_eq!(parser.find_val::<i64>("--bopt")?, &42i64);
/// }
/// {
///     pre_parser.add_opt("-d".infer::<String>())?.set_values(vec![]);
///     pre_parser.add_opt("--eopt=s")?;
///
///     let ret = getopt!(
///         Args::from(["-dbar", "-d", "foo", "--eopt=pre", "foo"]),
///         &mut pre_parser
///     )?;
///     let args = ret.ret.clone_args();
///     let parser = ret.parser;
///
///     assert_eq!(
///         parser.find_vals::<String>("-d")?,
///         &vec!["bar".to_owned(), "foo".to_owned()],
///     );
///     assert_eq!(parser.find_val::<String>("--eopt")?, &String::from("pre"));
///     assert_eq!(args, vec![OsStr::new("foo")] );
/// }
///
/// parser.reset()?;
/// pre_parser.reset()?;
///
/// {
///     let ret = getopt!(
///         Args::from(["-a", "--bopt=42", "foo"]),
///         "parser" => &mut parser,
///         "pre" => &mut pre_parser
///     )?;
///
///     assert_eq!(ret.parser, "parser");
///     assert_eq!(parser.find_val::<bool>("-a")?, &true);
///     assert_eq!(parser.find_val::<i64>("--bopt")?, &42i64);
/// }
/// {
///     let res = getopt!(
///         Args::from(["-dbar", "-d", "foo", "--eopt=pre", "foo"]),
///         "parser" => &mut parser,
///         "pre" => &mut pre_parser
///     )?;
///     let args = res.ret.clone_args();
///
///     assert_eq!(res.parser, "pre");
///     assert_eq!(
///         pre_parser.find_vals::<String>("-d")?,
///         &vec!["bar".to_owned(), "foo".to_owned()],
///     );
///     assert_eq!(pre_parser.find_val::<String>("--eopt")?, &String::from("pre"));
///     assert_eq!(args, vec![OsStr::new("foo")]);
/// }
/// # Ok(())
/// # }
///```
#[macro_export]
macro_rules! getopt {
    ($args:expr, $($parser_left:path),+) => {
        getopt!($args, $(&mut $parser_left)+)
    };
    ($args:expr, $(&mut $parser_left:path),+) => {
        {
            fn __check_p<'b, S: $crate::prelude::Set, P: $crate::prelude::Policy<Set = S, Error = $crate::Error>>
                (p: &'b mut $crate::prelude::Parser<S, P>) -> &'b mut $crate::prelude::Parser<S, P>
                { p }
            fn __check_a(a: $crate::prelude::Args) -> $crate::prelude::Args { a }

            let mut ret = $crate::Error::no_parser_matched();
            let args = __check_a($args);

            loop {
                $(
                    let parser = __check_p(&mut $parser_left);

                    match $crate::parser::Parser::parse(parser, args.clone()) {
                        Ok(mut parser_ret) => {
                            if let Some(error) = parser_ret.take_failure() {
                                ret = error;
                            }
                            else {
                                break Ok($crate::GetoptRes {
                                    ret: parser_ret,
                                    parser: parser,
                                });
                            }
                        }
                        Err(e) => {
                            ret = e;
                        }
                    }
                )+
                break Err(ret);
            }
        }
    };
    ($args:expr, $($parser_name:literal => $parser_left:path),+) => {
        getopt!($args, $($parser_name => &mut $parser_left)+)
    };
    ($args:expr, $($parser_name:literal => &mut $parser_left:path),+) => {
        {
            fn __check_p<'b, S: $crate::prelude::Set, P: $crate::prelude::Policy<Set = S, Error = $crate::Error>>
                (p: &'b mut $crate::prelude::Parser<S, P>) -> &'b mut $crate::prelude::Parser<S, P>
                { p }
            fn __check_a(a: $crate::prelude::Args) -> $crate::prelude::Args { a }

            let mut ret = $crate::Error::no_parser_matched();
            let args = __check_a($args);

            loop {
                $(
                    let parser = __check_p(&mut $parser_left);

                    match $crate::parser::Parser::parse(parser, args.clone()) {
                        Ok(mut parser_ret) => {
                            if let Some(error) = parser_ret.take_failure() {
                                ret = error;
                            }
                            else {
                                break Ok($crate::GetoptRes {
                                    ret: parser_ret,
                                    parser: $parser_name,
                                });
                            }
                        }
                        Err(e) => {
                            ret = e;
                        }
                    }
                )+
                break Err(ret);
            }
        }
    };
}

pub mod prelude {
    pub use crate::args::Args;
    pub use crate::ctx::wrap_handler;
    pub use crate::ctx::wrap_handler_action;
    pub use crate::ctx::wrap_handler_fallback_action;
    pub use crate::ctx::Ctx;
    pub use crate::ctx::HandlerCollection;
    pub use crate::ctx::InnerCtx;
    pub use crate::ctx::Invoker;
    pub use crate::ctx::NullStore;
    pub use crate::ctx::Store;
    pub use crate::ctx::VecStore;
    pub use crate::getopt;
    pub use crate::map::ErasedTy;
    pub use crate::opt::AOpt;
    pub use crate::opt::Action;
    pub use crate::opt::Cmd;
    pub use crate::opt::ConfigBuild;
    pub use crate::opt::ConfigBuildInfer;
    pub use crate::opt::ConfigBuildMutable;
    pub use crate::opt::ConfigBuildWith;
    pub use crate::opt::ConfigBuilder;
    pub use crate::opt::ConfigBuilderWith;
    pub use crate::opt::ConfigValue;
    pub use crate::opt::ConstrctInfo;
    pub use crate::opt::Creator;
    pub use crate::opt::Help;
    pub use crate::opt::Index;
    pub use crate::opt::Information;
    pub use crate::opt::Main;
    pub use crate::opt::MutOpt;
    pub use crate::opt::Opt;
    pub use crate::opt::OptConfig;
    pub use crate::opt::OptParser;
    pub use crate::opt::OptValueExt;
    pub use crate::opt::Pos;
    #[cfg(feature = "serde")]
    pub use crate::opt::Serde;
    pub use crate::opt::StrParser;
    pub use crate::opt::Style;
    pub use crate::parser::AppServices;
    pub use crate::parser::AppStorage;
    pub use crate::parser::DefaultSetChecker;
    pub use crate::parser::DelayPolicy;
    pub use crate::parser::FwdPolicy;
    pub use crate::parser::HCOptSet;
    pub use crate::parser::OptStyleManager;
    pub use crate::parser::Parser;
    pub use crate::parser::ParserCommit;
    pub use crate::parser::ParserCommitWithValue;
    pub use crate::parser::Policy;
    pub use crate::parser::PolicyParser;
    pub use crate::parser::PolicySettings;
    pub use crate::parser::Return;
    pub use crate::parser::SeqPolicy;
    pub use crate::parser::UserStyle;
    pub use crate::parser::UsrValService;
    pub use crate::set::ctor_default_name;
    pub use crate::set::Commit;
    pub use crate::set::Ctor;
    pub use crate::set::Filter;
    pub use crate::set::FilterMatcher;
    pub use crate::set::FilterMut;
    pub use crate::set::OptSet;
    pub use crate::set::OptValidator;
    pub use crate::set::PrefixOptValidator;
    pub use crate::set::PrefixedValidator;
    pub use crate::set::Set;
    pub use crate::set::SetCfg;
    pub use crate::set::SetChecker;
    pub use crate::set::SetCommit;
    pub use crate::set::SetCommitWithValue;
    pub use crate::set::SetExt;
    pub use crate::set::SetOpt;
    pub use crate::set::SetValueFindExt;
    pub use crate::value::AnyValue;
    pub use crate::value::ErasedValue;
    pub use crate::value::Infer;
    pub use crate::value::InitializeValue;
    pub use crate::value::RawValParser;
    pub use crate::value::ValAccessor;
    pub use crate::value::ValInitializer;
    pub use crate::value::ValStorer;
    pub use crate::value::ValValidator;
    pub use crate::ARef;
    pub use crate::GetoptRes;
    pub use crate::Uid;
    pub use std::ffi::OsStr;

    pub type ACreator = Creator<AOpt, OptConfig, crate::Error>;

    pub type ASet = OptSet<StrParser, ACreator, PrefixOptValidator>;

    pub type AHCSet<'a> = HCOptSet<'a, ASet>;

    pub type AInvoker<'a> = Invoker<'a, AHCSet<'a>>;

    pub type AFwdPolicy<'a> = FwdPolicy<AHCSet<'a>, DefaultSetChecker<AHCSet<'a>>>;

    pub type ADelayPolicy<'a> = DelayPolicy<AHCSet<'a>, DefaultSetChecker<AHCSet<'a>>>;

    pub type ASeqPolicy<'a> = SeqPolicy<AHCSet<'a>, DefaultSetChecker<AHCSet<'a>>>;

    pub type AFwdParser<'a> = Parser<AHCSet<'a>, AFwdPolicy<'a>>;

    pub type ADelayParser<'a> = Parser<AHCSet<'a>, ADelayPolicy<'a>>;

    pub type ASeqParser<'a> = Parser<AHCSet<'a>, ASeqPolicy<'a>>;
}
