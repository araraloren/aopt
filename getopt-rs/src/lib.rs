
pub mod arg;
pub mod str;
pub mod err;
pub mod opt;

pub(crate) mod pattern;

#[macro_use]
extern crate log;

pub mod tools {
    use simplelog::{Config, CombinedLogger, SimpleLogger};
    use log::LevelFilter;

    pub fn initialize_log() -> std::result::Result<(), log::SetLoggerError> {
        CombinedLogger::init(vec![
            SimpleLogger::new(LevelFilter::Warn, Config::default()),
            SimpleLogger::new(LevelFilter::Error, Config::default()),
            SimpleLogger::new(LevelFilter::Debug, Config::default()),
            SimpleLogger::new(LevelFilter::Info, Config::default()),
            SimpleLogger::new(LevelFilter::Trace, Config::default()),
        ])
    }
}