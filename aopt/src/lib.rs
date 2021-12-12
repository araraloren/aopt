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
use crate::err::Result;
use crate::parser::Parser;
use crate::set::Set;

pub fn gstr(s: &str) -> ustr::Ustr {
    ustr::Ustr::from(s)
}

#[derive(Debug)]
pub struct ReturnValue<'a, 'b>(pub &'b mut dyn Parser, pub &'a mut dyn Set);

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
