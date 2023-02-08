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
pub type Arc<T> = std::sync::Arc<T>;
#[cfg(not(feature = "sync"))]
pub type Arc<T> = std::rc::Rc<T>;

#[cfg(feature = "log")]
pub(crate) use tracing::trace as trace_log;
#[cfg(not(feature = "log"))]
#[macro_use]
pub(crate) mod log {
    #[macro_export]
    macro_rules! trace_log {
        ($($_:stmt),+) => {};
    }
}

pub use crate::err::Error;
pub use crate::err::Result;
pub use crate::str::astr;
pub use crate::str::Str;
pub use crate::str::StrJoin;

use std::any::TypeId;

/// Get the [`TypeId`](std::any::TypeId) of type `T`.
pub(crate) fn typeid<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

use parser::Parser;
use parser::Policy;
use parser::ReturnVal;

pub struct GetoptRes<'a, P: Policy> {
    pub ret: ReturnVal,

    pub parser: &'a mut Parser<P>,
}

/// Parse the given string sequence, return the first matched [`Parser`](crate::parser::Parser): `getopt!($args, $($parser),+)`.
///
/// # Returns
///
/// Will return an Ok(Some([`Parser`](crate::parser::Parser))) if any [`Parser`](crate::parser::Parser) parsing successed, otherwise return `Ok(None)`.
///
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
///     let ret = getopt!(Args::from_array(["-a", "--bopt=42", "foo"]), &mut parser)?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///     let ret = ret.parser;
///
///     assert_eq!(ret.find_val::<bool>("-a")?, &true);
///     assert_eq!(ret.find_val::<i64>("--bopt")?, &42i64);
/// }
/// {
///     pre_parser.add_opt("-d=s")?;
///     pre_parser.add_opt("--eopt=s")?;
///
///     let ret = getopt!(
///         Args::from_array(["-dbar", "-d", "foo", "--eopt=pre", "foo"]),
///         &mut pre_parser
///     )?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///     let args = ret.ret.clone_args();
///     let ret = ret.parser;
///
///     assert_eq!(
///         ret.find_vals::<String>("-d")?,
///         &vec!["bar".to_owned(), "foo".to_owned()],
///     );
///     assert_eq!(ret.find_val::<String>("--eopt")?, &String::from("pre"));
///     assert_eq!(args, vec![RawVal::from("foo")] );
/// }
///
/// parser.reset()?;
/// pre_parser.reset()?;
///
/// // boxed it
/// let mut parser = parser.into_boxed();
/// let mut pre_parser = pre_parser.into_boxed();
///
/// {
///     let ret = getopt!(
///         Args::from_array(["-a", "--bopt=42", "foo"]),
///         &mut parser,
///         &mut pre_parser
///     )?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///     let ret = ret.parser;
///
///     assert_eq!(ret.find_val::<bool>("-a")?, &true);
///     assert_eq!(ret.find_val::<i64>("--bopt")?, &42i64);
/// }
/// {
///     let ret = getopt!(
///         Args::from_array(["-dbar", "-d", "foo", "--eopt=pre", "foo"]),
///         &mut parser,
///         &mut pre_parser
///     )?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///     let args = ret.ret.clone_args();
///     let ret = ret.parser;
///
///     assert_eq!(
///         ret.find_vals::<String>("-d")?,
///         &vec!["bar".to_owned(), "foo".to_owned()],
///     );
///     assert_eq!(ret.find_val::<String>("--eopt")?, &String::from("pre"));
///     assert_eq!(args, vec![RawVal::from("foo")]);
/// }
/// # Ok(())
/// # }
///```
#[macro_export]
macro_rules! getopt {
    ($args:expr, $($parser_left:expr),+) => {
        {
            fn __check_p<P: $crate::prelude::Policy<Error = $crate::Error>>
                (p: &mut $crate::prelude::Parser<P>) -> &mut $crate::prelude::Parser<P>
                { p }
            fn __check_a(a: $crate::prelude::Args) -> $crate::prelude::Args { a }

            let mut ret = Ok(None);
            let args = $crate::Arc::new(__check_a($args));

            loop {
                $(
                    let parser = __check_p($parser_left);

                    parser.init()?;
                    match parser.parse(args.clone()) {
                        Ok(mut parser_ret) => {
                            if parser_ret.status() {
                                break Ok(Some($crate::GetoptRes {
                                    ret: parser_ret,
                                    parser,
                                }));
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
    pub use crate::ctx::wrap_handler_fallback;
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
    pub use crate::opt::Config;
    pub use crate::opt::ConfigValue;
    pub use crate::opt::ConstrctInfo;
    pub use crate::opt::Creator;
    pub use crate::opt::Help;
    pub use crate::opt::Index;
    pub use crate::opt::Information;
    pub use crate::opt::Noa;
    pub use crate::opt::Opt;
    pub use crate::opt::OptConfig;
    pub use crate::opt::OptParser;
    pub use crate::opt::OptValueExt;
    #[cfg(feature = "serde")]
    pub use crate::opt::Serde;
    pub use crate::opt::StrParser;
    pub use crate::opt::Style;
    pub use crate::parser::BoxedPolicy;
    pub use crate::parser::DelayPolicy;
    pub use crate::parser::FwdPolicy;
    pub use crate::parser::OptStyleManager;
    pub use crate::parser::Parser;
    pub use crate::parser::ParserCommit;
    pub use crate::parser::ParserCommitWithValue;
    pub use crate::parser::Policy;
    pub use crate::parser::PrePolicy;
    pub use crate::parser::ReturnVal;
    pub use crate::parser::SetChecker;
    pub use crate::parser::UserStyle;
    pub use crate::parser::UserStyleManager;
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
    pub use crate::set::SetCommit;
    pub use crate::set::SetCommitWithValue;
    pub use crate::set::SetExt;
    pub use crate::set::SetOpt;
    pub use crate::set::SetValueFindExt;
    pub use crate::value::AnyValue;
    pub use crate::value::ErasedValHandler;
    pub use crate::value::Infer;
    pub use crate::value::InitializeValue;
    pub use crate::value::RawValParser;
    pub use crate::value::ValAccessor;
    pub use crate::value::ValInitializer;
    pub use crate::value::ValStorer;
    pub use crate::value::ValValidator;
    pub use crate::Uid;
}
