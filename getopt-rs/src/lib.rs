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
extern crate log;

pub mod tools {
    use crate::opt::{nonopt, opt};
    pub use crate::opt::{
        Alias, Callback, Help, HelpInfo, Identifier, Index, Name, Opt, OptCallback, OptIndex,
        OptValue, Optional, Type, Value,
    };
    use crate::set::Set;
    use log::LevelFilter;
    use simplelog::{CombinedLogger, Config, SimpleLogger};

    pub fn initialize_log() -> std::result::Result<(), log::SetLoggerError> {
        CombinedLogger::init(vec![
            SimpleLogger::new(LevelFilter::Warn, Config::default()),
            SimpleLogger::new(LevelFilter::Error, Config::default()),
            SimpleLogger::new(LevelFilter::Debug, Config::default()),
            SimpleLogger::new(LevelFilter::Info, Config::default()),
            //SimpleLogger::new(LevelFilter::Trace, Config::default()),
        ])
    }

    pub fn initialize_creator<S: Set>(set: &mut S) {
        set.add_creator(Box::new(opt::ArrayCreator::default()));
        set.add_creator(Box::new(opt::BoolCreator::default()));
        set.add_creator(Box::new(opt::FltCreator::default()));
        set.add_creator(Box::new(opt::IntCreator::default()));
        set.add_creator(Box::new(opt::StrCreator::default()));
        set.add_creator(Box::new(opt::UintCreator::default()));
        set.add_creator(Box::new(nonopt::CmdCreator::default()));
        set.add_creator(Box::new(nonopt::MainCreator::default()));
        set.add_creator(Box::new(nonopt::PosCreator::default()));
    }

    pub fn initialize_prefix<S: Set>(set: &mut S) {
        set.add_prefix(String::from("--"));
        set.add_prefix(String::from("-"));
    }

    #[macro_export]
    macro_rules! simple_main_cb {
        ($block:expr) => {
            getopt_rs::opt::callback::Callback::Main(Box::new(
                getopt_rs::opt::callback::SimpleMainCallback::new($block),
            ))
        };
    }

    #[macro_export]
    macro_rules! simple_main_mut_cb {
        ($block:expr) => {
            getopt_rs::opt::callback::Callback::MainMut(Box::new(
                getopt_rs::opt::callback::SimpleMainMutCallback::new($block),
            ))
        };
    }

    #[macro_export]
    macro_rules! simple_pos_cb {
        ($block:expr) => {
            getopt_rs::opt::callback::Callback::Pos(Box::new(
                getopt_rs::opt::callback::SimplePosCallback::new($block),
            ))
        };
    }

    #[macro_export]
    macro_rules! simple_pos_mut_cb {
        ($block:expr) => {
            getopt_rs::opt::callback::Callback::PosMut(Box::new(
                getopt_rs::opt::callback::SimplePosMutCallback::new($block),
            ))
        };
    }

    #[macro_export]
    macro_rules! simple_opt_cb {
        ($block:expr) => {
            getopt_rs::opt::callback::Callback::Opt(Box::new(
                getopt_rs::opt::callback::SimpleOptCallback::new($block),
            ))
        };
    }

    #[macro_export]
    macro_rules! simple_opt_mut_cb {
        ($block:expr) => {
            getopt_rs::opt::callback::Callback::OptMut(Box::new(
                getopt_rs::opt::callback::SimpleOptMutCallback::new($block),
            ))
        };
    }
}

pub mod prelude {
    pub use crate::ctx::{Context, NonOptContext, OptContext};
    pub use crate::err::{Error, Result};
    pub use crate::opt::callback::{SimpleMainCallback, SimpleMainMutCallback};
    pub use crate::opt::callback::{SimpleOptCallback, SimpleOptMutCallback};
    pub use crate::opt::callback::{SimplePosCallback, SimplePosMutCallback};
    pub use crate::opt::{nonopt, opt};
    pub use crate::opt::{
        Alias, Callback, Help, HelpInfo, Identifier, Index, Name, Opt, OptCallback, OptIndex,
        OptValue, Optional, Type, Value,
    };
    pub use crate::parser::{Parser, SimpleParser};
    pub use crate::proc::{Info, Proc};
    pub use crate::proc::{Matcher, NonOptMatcher, OptMatcher};
    pub use crate::set::{CreatorSet, OptionSet, PrefixSet, Set, SimpleSet};
    pub use crate::tools;
    pub use crate::uid::{Uid, UidGenerator};
    pub use crate::{simple_main_cb, simple_main_mut_cb};
    pub use crate::{simple_opt_cb, simple_opt_mut_cb};
    pub use crate::{simple_pos_cb, simple_pos_mut_cb};
}
