#![doc = include_str!("../README.md")]
pub mod args;
pub mod ctx;
pub mod err;
pub mod ext;
pub mod map;
pub mod opt;
pub mod parser;
pub mod proc;
pub mod raw;
pub mod ser;
pub mod set;
pub mod str;
pub mod value;

pub type Uid = u64;
pub type HashMap<K, V> = ahash::HashMap<K, V>;
pub type RawVal = raw::RawVal;

#[cfg(feature = "sync")]
pub type ARef<T> = std::sync::Arc<T>;
#[cfg(not(feature = "sync"))]
pub type ARef<T> = std::rc::Rc<T>;

#[cfg(feature = "log")]
pub(crate) use tracing::trace as trace_log;
#[cfg(not(feature = "log"))]
#[macro_use]
pub(crate) mod log {
    #[macro_export]
    macro_rules! trace_log {
        ($($arg:tt)*) => {};
    }
}

pub(crate) fn display_option<T: Display>(option_value: &Option<T>) -> String {
    if let Some(value) = option_value {
        format!("Some({value})")
    } else {
        "None".to_owned()
    }
}

pub use crate::err::Error;
pub use crate::err::Result;
pub use crate::str::astr;
pub use crate::str::Str;
pub use crate::str::StrJoin;

use std::any::TypeId;
use std::fmt::Display;

/// Get the [`TypeId`](std::any::TypeId) of type `T`.
pub(crate) fn typeid<T: ?Sized + 'static>() -> TypeId {
    TypeId::of::<T>()
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
/// Will return Err([`Error::default()`]) if all [`Parser`](crate::parser::Parser) parsing failed, otherwise return Err(_).
/// # Example
///
/// ```rust
/// # use aopt::err::Result;
/// # use aopt::{prelude::*, RawVal};
/// #
/// # fn main() -> Result<()> {
/// let mut parser = AFwdParser::default();
/// let mut pre_parser = APreParser::default();
///
/// {
///     parser.add_opt("-a=b!")?;
///     parser.add_opt("--bopt=i")?;
///     parser.add_opt("c=p@-0")?.on(
///         |_: &mut ASet, _: &mut ASer, args: ctx::Args, mut val: ctx::Value<String>| {
///             assert_eq!(args[0], RawVal::from("foo"));
///             Ok(Some(val.take()))
///         },
///     )?;
///
///     assert!(getopt!(Args::from_array(["-a", "--bopt=42", "foo"]), &mut parser).is_ok());
///     assert_eq!(parser.find_val::<bool>("-a")?, &true);
///     assert_eq!(parser.find_val::<i64>("--bopt")?, &42i64);
/// }
/// {
///     pre_parser.add_opt_i::<String>("-d")?.set_values(vec![]);
///     pre_parser.add_opt("--eopt=s")?;
///
///     let ret = getopt!(
///         Args::from_array(["-dbar", "-d", "foo", "--eopt=pre", "foo"]),
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
///     assert_eq!(args, vec![RawVal::from("foo")] );
/// }
///
/// parser.reset()?;
/// pre_parser.reset()?;
///
/// {
///     let ret = getopt!(
///         Args::from_array(["-a", "--bopt=42", "foo"]),
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
///         Args::from_array(["-dbar", "-d", "foo", "--eopt=pre", "foo"]),
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
///     assert_eq!(args, vec![RawVal::from("foo")]);
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
            fn __check_p<'a, 'b, P: $crate::prelude::Policy<Error = $crate::Error>>
                (p: &'b mut $crate::prelude::Parser<'a, P>) -> &'b mut $crate::prelude::Parser<'a, P>
                { p }
            fn __check_a(a: $crate::prelude::Args) -> $crate::prelude::Args { a }

            let mut ret = Err($crate::err::Error::default());
            let args = $crate::ARef::new(__check_a($args));

            loop {
                $(
                    let parser = __check_p(&mut $parser_left);

                    parser.init()?;
                    match parser.parse(args.clone()) {
                        Ok(mut parser_ret) => {
                            if parser_ret.status() {
                                break Ok($crate::GetoptRes {
                                    ret: parser_ret,
                                    parser: parser,
                                });
                            }
                            else {
                                ret = Err(parser_ret.take_failure());
                            }
                        }
                        Err(e) => {
                            ret = Err(e);
                        }
                    }
                )+
                break ret;
            }
        }
    };
    ($args:expr, $($parser_name:literal => $parser_left:path),+) => {
        getopt!($args, $($parser_name => &mut $parser_left)+)
    };
    ($args:expr, $($parser_name:literal => &mut $parser_left:path),+) => {
        {
            fn __check_p<'a, 'b, P: $crate::prelude::Policy<Error = $crate::Error>>
                (p: &'b mut $crate::prelude::Parser<'a, P>) -> &'b mut $crate::prelude::Parser<'a, P>
                { p }
            fn __check_a(a: $crate::prelude::Args) -> $crate::prelude::Args { a }

            let mut ret = Err($crate::err::Error::default());
            let args = $crate::ARef::new(__check_a($args));

            loop {
                $(
                    let parser = __check_p(&mut $parser_left);

                    parser.init()?;
                    match parser.parse(args.clone()) {
                        Ok(mut parser_ret) => {
                            if parser_ret.status() {
                                break Ok($crate::GetoptRes {
                                    ret: parser_ret,
                                    parser: $parser_name,
                                });
                            }
                            else {
                                ret = Err(parser_ret.take_failure());
                            }
                        }
                        Err(e) => {
                            ret = Err(e);
                        }
                    }
                )+
                break ret;
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
    pub use crate::ctx::Extract;
    pub use crate::ctx::Handler;
    pub use crate::ctx::InnerCtx;
    pub use crate::ctx::Invoker;
    pub use crate::ctx::NullStore;
    pub use crate::ctx::Store;
    pub use crate::ctx::VecStore;
    pub use crate::ext::*;
    pub use crate::getopt;
    pub use crate::map::ErasedTy;
    pub use crate::opt::AOpt;
    pub use crate::opt::Action;
    pub use crate::opt::Cmd;
    pub use crate::opt::Config;
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
    pub use crate::opt::RefOpt;
    #[cfg(feature = "serde")]
    pub use crate::opt::Serde;
    pub use crate::opt::StrParser;
    pub use crate::opt::Style;
    pub use crate::value::InferValueMut;
    pub use crate::value::InferValueRef;
    // pub use crate::parser::BoxedPolicy;
    pub use crate::ctx::HandlerCollection;
    pub use crate::parser::DefaultSetChecker;
    pub use crate::parser::DelayPolicy;
    pub use crate::parser::FwdPolicy;
    pub use crate::parser::OptStyleManager;
    pub use crate::parser::Parser;
    pub use crate::parser::ParserCommit;
    pub use crate::parser::ParserCommitWithValue;
    pub use crate::parser::Policy;
    pub use crate::parser::PolicySettings;
    pub use crate::parser::PrePolicy;
    pub use crate::parser::ReturnVal;
    pub use crate::parser::UserStyle;
    pub use crate::proc::Match;
    pub use crate::proc::NOAMatch;
    pub use crate::proc::NOAProcess;
    pub use crate::proc::OptMatch;
    pub use crate::proc::OptProcess;
    pub use crate::proc::Process;
    pub use crate::ser::AppServices;
    pub use crate::ser::ServicesExt;
    pub use crate::ser::ServicesValExt;
    pub use crate::ser::UsrValService;
    pub use crate::set::ctor_default_name;
    pub use crate::set::Commit;
    pub use crate::set::Ctor;
    pub use crate::set::Filter;
    pub use crate::set::FilterMatcher;
    pub use crate::set::FilterMut;
    pub use crate::set::OptSet;
    pub use crate::set::OptValidator;
    pub use crate::set::PrefixOptValidator;
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
    pub use crate::RawVal;
    pub use crate::Uid;
}
