pub mod cmd;
pub mod main;
pub mod pos;

use crate::opt::Opt;
pub trait NonOpt: Opt {}
