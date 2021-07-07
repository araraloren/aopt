pub mod cmd;
pub mod main;
pub mod pos;

use crate::opt::Opt;
pub trait NonOpt: Opt {}

pub use cmd::{CmdCreator, CmdOpt};
pub use main::{MainCreator, MainOpt};
pub use pos::{PosCreator, PosOpt};
