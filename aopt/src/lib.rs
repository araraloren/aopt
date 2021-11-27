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

use crate::err::Result;
use crate::parser::Parser;
use crate::set::Set;

// declare a alias for string type
pub use ustr::Ustr;
pub use ustr::UstrMap;
pub use ustr::UstrSet;

pub fn gstr(s: &str) -> Ustr {
    Ustr::from(s)
}

pub struct ReturnValue<'a, 'b>(&'b mut dyn Parser, &'a mut dyn Set);

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
        match parser.parse(set, &mut args.iter().map(|v| v.clone())) {
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
    mut iter: impl Iterator<Item = String>,
    set: &'a mut dyn Set,
    parser: &'b mut dyn Parser,
) -> Result<Option<ReturnValue<'a, 'b>>> {
    if parser.parse(set, &mut iter)? {
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

pub mod tools {
    #[macro_export]
    macro_rules! simple_main_cb {
        ($block:expr) => {
            OptCallback::Main(Box::new(SimpleMainCallback::new($block)))
        };
    }

    #[macro_export]
    macro_rules! simple_main_mut_cb {
        ($block:expr) => {
            OptCallback::MainMut(Box::new(SimpleMainMutCallback::new($block)))
        };
    }

    #[macro_export]
    macro_rules! simple_pos_cb {
        ($block:expr) => {
            OptCallback::Pos(Box::new(SimplePosCallback::new($block)))
        };
    }

    #[macro_export]
    macro_rules! simple_pos_mut_cb {
        ($block:expr) => {
            OptCallback::PosMut(Box::new(SimplePosMutCallback::new($block)))
        };
    }

    #[macro_export]
    macro_rules! simple_opt_cb {
        ($block:expr) => {
            OptCallback::Opt(Box::new(SimpleOptCallback::new($block)))
        };
    }

    #[macro_export]
    macro_rules! simple_opt_mut_cb {
        ($block:expr) => {
            OptCallback::OptMut(Box::new(SimpleOptMutCallback::new($block)))
        };
    }
}

pub mod prelude {
    pub use crate::ctx::{Context, NonOptContext, OptContext};
    pub use crate::gstr;
    pub use crate::opt::callback::{SimpleMainCallback, SimpleMainMutCallback};
    pub use crate::opt::callback::{SimpleOptCallback, SimpleOptMutCallback};
    pub use crate::opt::callback::{SimplePosCallback, SimplePosMutCallback};
    pub use crate::opt::{
        Alias, Callback, Help, HelpInfo, Identifier, Index, Name, Opt, OptCallback, OptIndex,
        OptValue, Optional, Type, Value,
    };
    pub use crate::opt::{
        ArrayCreator, BoolCreator, FltCreator, IntCreator, StrCreator, UintCreator,
    };
    pub use crate::opt::{CmdCreator, MainCreator, PosCreator};
    pub use crate::parser::{DelayParser, Parser, PreParser, SimpleParser};
    pub use crate::proc::{Info, Proc};
    pub use crate::proc::{Matcher, NonOptMatcher, OptMatcher};
    pub use crate::set::{CreatorSet, OptionSet, PrefixSet, Set, SimpleSet};
    pub use crate::tools;
    pub use crate::uid::{Uid, UidGenerator};
    pub use crate::{getopt, getopt_impl, getopt_impl_s, ReturnValue};
    pub use crate::{simple_main_cb, simple_main_mut_cb};
    pub use crate::{simple_opt_cb, simple_opt_mut_cb};
    pub use crate::{simple_pos_cb, simple_pos_mut_cb};
    pub use crate::{Ustr, UstrMap};
}
