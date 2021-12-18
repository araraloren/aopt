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
use crate::parser::Parser;
use crate::set::Set;

pub use crate::err::Error;
pub use crate::err::Result;

/// Create a [`Ustr`](ustr::Ustr) from `&str`.
pub fn gstr(s: &str) -> ustr::Ustr {
    ustr::Ustr::from(s)
}

/// The return value of [`getopt!`].
#[derive(Debug)]
pub struct ReturnValue<'a, 'b>(pub &'b mut dyn Parser, pub &'a mut dyn Set);

impl<'a, 'b> ReturnValue<'a, 'b> {
    /// Get the parser of return value.
    pub fn parser(&self) -> &dyn Parser {
        self.0
    }

    /// Get the set of return value.
    pub fn set(&self) -> &dyn Set {
        self.1
    }

    /// Get the parser mutable reference of return value.
    pub fn parser_mut(&mut self) -> &mut dyn Parser {
        self.0
    }

    /// Get the set mutable reference of return value.
    pub fn set_mut(&mut self) -> &mut dyn Set {
        self.1
    }
}

pub fn getopt_impl<'a, 'b>(
    iter: impl Iterator<Item = String>,
    sets: Vec<&'a mut dyn Set>,
    parsers: Vec<&'b mut dyn Parser>,
) -> Result<Option<ReturnValue<'a, 'b>>> {
    assert_eq!(sets.len(), parsers.len());

    let args: Vec<String> = iter.collect();
    let count = parsers.len();
    let mut index = 0;

    for (parser, set) in parsers.into_iter().zip(sets.into_iter()) {
        let mut stream = ArgStream::from(args.clone().into_iter());

        match parser.parse(set, &mut stream) {
            Ok(rv) => {
                if rv {
                    return Ok(Some(ReturnValue(parser, set)));
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

pub fn getopt_impl_s<'a, 'b>(
    iter: impl Iterator<Item = String>,
    set: &'a mut dyn Set,
    parser: &'b mut dyn Parser,
) -> Result<Option<ReturnValue<'a, 'b>>> {
    let mut stream = ArgStream::from(iter);

    if parser.parse(set, &mut stream)? {
        return Ok(Some(ReturnValue(parser, set)));
    } else {
        Ok(None)
    }
}

/// Parse the given string sequence, return the first matched [`Parser`] and [`Set`].
///
/// # Returns
///
/// Will return an Some([`ReturnValue`]) if any [`Parser`] parsing successed, otherwise return None.  
///
/// # Example
///
/// ```rust
/// use aopt::prelude::*;
/// use aopt::err::Result;
///
/// fn main() -> Result<()> {
///     let mut parser = SimpleParser::<UidGenerator>::default();
///     let mut pre_parser = PreParser::<UidGenerator>::default();
///     let mut set = SimpleSet::default()
///         .with_default_creator()
///         .with_default_prefix();
///     let mut pre_set = SimpleSet::default()
///         .with_default_creator()
///         .with_default_prefix();
///     set.add_opt("-a=b!")?.commit()?;
///     set.add_opt("--bopt=i")?.commit()?;
///     parser.add_callback(
///         set.add_opt("c=p@-1")?.commit()?,
///         simple_pos_cb!(|_, _, arg, _, value| {
///             assert_eq!(arg, "foo");
///             Ok(Some(value))
///         }),
///     );
///     pre_set.add_opt("-d=a")?.commit()?;
///     pre_set.add_opt("--eopt=s")?.commit()?;
///     {
///         let ret = getopt!(
///             &mut ["-a", "--bopt=42", "foo"].iter().map(|&v| String::from(v)),
///             set,
///             parser,
///             pre_set,
///             pre_parser
///         )?;

///         assert!(ret.is_some());
///         assert_eq!(
///             ret.as_ref().unwrap().set().get_value("-a")?,
///             Some(&OptValue::from(true))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().set().get_value("--bopt")?,
///             Some(&OptValue::from(42i64))
///         );
///     }
///     {
///         let ret = getopt!(
///             &mut ["-dbar", "-d", "foo", "--eopt=pre", "foo"]
///                 .iter()
///                 .map(|&v| String::from(v)),
///             set,
///             parser,
///             pre_set,
///             pre_parser
///         )?;
///         assert!(ret.is_some());
///         assert_eq!(
///             ret.as_ref().unwrap().set().get_value("-d")?,
///             Some(&OptValue::from(vec!["bar".to_owned(), "foo".to_owned()]))
///         );
///         assert_eq!(
///             ret.as_ref().unwrap().set().get_value("--eopt")?,
///             Some(&OptValue::from("pre"))
///         );
///         assert_eq!(ret.as_ref().unwrap().parser().get_noa(), ["foo"]);
///     }
///     {
///         assert!(getopt!(
///             &mut ["-dbar", "-d", "foo", "--eopt=pre", "foo"]
///                 .iter()
///                 .map(|&v| String::from(v)),
///             set,
///             parser
///         )
///         .is_err());
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! getopt {
    ($iter:expr, $set:expr, $parser:expr ) => {
        getopt_impl_s(
            $iter,
            &mut $set,
            &mut $parser
        )
    };
    ($iter:expr, $($set:expr, $parser:expr),+ ) => {
        getopt_impl(
            $iter,
            vec![$(&mut $set, )+],
            vec![$(&mut $parser, )+]
        )
    };
}

pub mod prelude {
    pub use crate::ctx::Context;
    pub use crate::ctx::NonOptContext;
    pub use crate::ctx::OptContext;
    pub use crate::getopt;
    pub use crate::getopt_impl;
    pub use crate::getopt_impl_s;
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
    pub use crate::parser::DelayParser;
    pub use crate::parser::Parser;
    pub use crate::parser::PreParser;
    pub use crate::parser::SimpleParser;
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
    pub use crate::ReturnValue;
    pub use ustr::Ustr;
    pub use ustr::UstrMap;
}
