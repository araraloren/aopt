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

pub type Uid = u64;
pub type HashMap<K, V> = ahash::HashMap<K, V>;
pub type RawVal = raw::RawVal;
cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub type Arc<T> = std::sync::Arc<T>;
    }
    else {
        pub type Arc<T> = std::rc::Rc<T>;
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
///     parser.add_opt("c=p@-1")?.on(
///         |_: &mut ASet, _: &mut ASer, args: ctx::Args, mut val: ctx::Value<String>| {
///             assert_eq!(args[0], RawVal::from("foo"));
///             Ok(Some(val.take()))
///         },
///     )?;
///
///     let ret = getopt!(["-a", "--bopt=42", "foo"].into_iter(), &mut parser)?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///
///     assert_eq!(ret.find_val::<bool>("-a")?, &true);
///     assert_eq!(ret.find_val::<i64>("--bopt")?, &42i64);
/// }
/// {
///     pre_parser.add_opt("-d=s")?;
///     pre_parser.add_opt("--eopt=s")?;
///
///     let ret = getopt!(
///         ["-dbar", "-d", "foo", "--eopt=pre", "foo"].into_iter(),
///         &mut pre_parser
///     )?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///
///     assert_eq!(
///         ret.find_vals::<String>("-d")?,
///         &vec!["bar".to_owned(), "foo".to_owned()],
///     );
///     assert_eq!(ret.find_val::<String>("--eopt")?, &String::from("pre"));
///     assert_eq!(
///         ret.take_retval().unwrap().take_args(),
///         vec![RawVal::from("foo")]
///     );
/// }
///
/// parser.clear_all()?;
/// pre_parser.clear_all()?;
///
/// // boxed it
/// let mut parser = parser.into_boxed();
/// let mut pre_parser = pre_parser.into_boxed();
///
/// {
///     let ret = getopt!(
///         ["-a", "--bopt=42", "foo"].into_iter(),
///         &mut parser,
///         &mut pre_parser
///     )?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///
///     assert_eq!(ret.find_val::<bool>("-a")?, &true);
///     assert_eq!(ret.find_val::<i64>("--bopt")?, &42i64);
/// }
/// {
///     let ret = getopt!(
///         ["-dbar", "-d", "foo", "--eopt=pre", "foo"].into_iter(),
///         &mut parser,
///         &mut pre_parser
///     )?;
///
///     assert!(ret.is_some());
///     let ret = ret.unwrap();
///
///     assert_eq!(
///         ret.find_vals::<String>("-d")?,
///         &vec!["bar".to_owned(), "foo".to_owned()],
///     );
///     assert_eq!(ret.find_val::<String>("--eopt")?, &String::from("pre"));
///     assert_eq!(
///         ret.take_retval().unwrap().take_args(),
///         vec![RawVal::from("foo")]
///     );
/// }
/// # Ok(())
/// # }
///```
#[macro_export]
macro_rules! getopt {
    ($args:expr, $($parser_left:expr),+) => {
        {
            fn __check_p<P: $crate::prelude::Policy<Error = $crate::Error>>
                (p: &mut $crate::prelude::Parser<P>) -> &mut $crate::prelude::Parser<P> { p }

            let mut ret = Ok(None);
            let args = $crate::Arc::new($crate::prelude::Args::new($args));

            loop {
                $(
                    let parser = __check_p($parser_left);

                    parser.init()?;
                    match parser.parse(args.clone()) {
                        Ok(parse_ret) => {
                            if let Some(_) = parse_ret {
                                break Ok(Some(parser));
                            }
                            else { ret = Ok(None); }
                        }
                        Err(e) => {
                            if ! e.is_failure() {
                                break Err(e);
                            }
                            else { ret = Err(e); }
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
    pub use crate::ctx::Invoker;
    pub use crate::ctx::Store;
    pub use crate::ext::*;
    pub use crate::getopt;
    pub use crate::map::ErasedTy;
    pub use crate::opt::AOpt;
    pub use crate::opt::Action;
    pub use crate::opt::Assoc;
    pub use crate::opt::Config;
    pub use crate::opt::ConfigValue;
    pub use crate::opt::ConstrctInfo;
    pub use crate::opt::Creator;
    pub use crate::opt::Help;
    pub use crate::opt::Index;
    pub use crate::opt::Information;
    pub use crate::opt::Opt;
    pub use crate::opt::OptConfig;
    pub use crate::opt::OptParser;
    pub use crate::opt::RawValParser;
    pub use crate::opt::RawValValidator;
    pub use crate::opt::Serde;
    pub use crate::opt::StrParser;
    pub use crate::opt::Style;
    pub use crate::opt::ValInitialize;
    pub use crate::opt::ValInitiator;
    pub use crate::opt::ValValidator;
    pub use crate::parser::DelayPolicy;
    pub use crate::parser::FwdPolicy;
    pub use crate::parser::Parser;
    pub use crate::parser::ParserCommit;
    pub use crate::parser::Policy;
    pub use crate::parser::PrePolicy;
    pub use crate::proc::Match;
    pub use crate::proc::NOAMatch;
    pub use crate::proc::NOAProcess;
    pub use crate::proc::OptMatch;
    pub use crate::proc::OptProcess;
    pub use crate::proc::Process;
    pub use crate::ser::AnyValEntry;
    pub use crate::ser::AnyValService;
    pub use crate::ser::RawValService;
    pub use crate::ser::Services;
    pub use crate::ser::ServicesExt;
    pub use crate::ser::ServicesValExt;
    pub use crate::ser::UsrValService;
    pub use crate::set::Commit;
    pub use crate::set::Ctor;
    pub use crate::set::Filter;
    pub use crate::set::FilterMatcher;
    pub use crate::set::FilterMut;
    pub use crate::set::OptSet;
    pub use crate::set::OptValidator;
    pub use crate::set::PrefixOptValidator;
    pub use crate::set::Set;
    pub use crate::set::SetExt;
    pub use crate::Uid;
}
