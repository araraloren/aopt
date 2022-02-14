pub mod app;
pub mod arg;
pub mod ctx;
pub mod err;
pub mod opt;
pub mod parser;
pub mod proc;
pub mod set;
pub mod uid;

pub(crate) mod pat;

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

pub fn getopt_dynparser<'a, I, ITER, S, SS>(
    iter: ITER,
    parsers: Vec<&'a mut DynParser<S, SS>>,
) -> Result<Option<&'a mut DynParser<S, SS>>>
where
    I: Into<String>,
    ITER: Iterator<Item = I>,
    S: Set,
    SS: Service,
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

pub fn getopt_parser<'a, I, ITER, S, SS, P>(
    iter: ITER,
    parsers: Vec<&'a mut Parser<S, SS, P>>,
) -> Result<Option<&'a mut Parser<S, SS, P>>>
where
    I: Into<String>,
    ITER: Iterator<Item = I>,
    S: Set,
    SS: Service,
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

/// Parse the given string sequence, return the first matched [`DynParser`].
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

/// Parse the given string sequence, return the first matched [`Parser`].
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

pub mod prelude {
    pub use crate::arg::ArgStream;
    pub use crate::ctx::Context;
    pub use crate::ctx::NonOptContext;
    pub use crate::ctx::OptContext;
    pub use crate::getopt;
    pub use crate::getopt_dynparser;
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
    pub use crate::opt::SimpleMainCallback;
    pub use crate::opt::SimpleMainMutCallback;
    pub use crate::opt::SimpleOptCallback;
    pub use crate::opt::SimpleOptMutCallback;
    pub use crate::opt::SimplePosCallback;
    pub use crate::opt::SimplePosMutCallback;
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
    pub use crate::proc::Info;
    pub use crate::proc::Matcher;
    pub use crate::proc::NonOptMatcher;
    pub use crate::proc::OptMatcher;
    pub use crate::proc::Proc;
    pub use crate::set::CreatorSet;
    pub use crate::set::OptionSet;
    pub use crate::set::PrefixSet;
    pub use crate::set::Set;
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
