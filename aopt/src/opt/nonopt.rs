mod cmd;
mod main;
mod pos;

use crate::opt::Opt;
pub trait NonOpt: Opt {}

pub use self::cmd::{Cmd, CmdCreator, CmdOpt};
pub use self::main::{Main, MainCreator, MainOpt};
pub use self::pos::{Pos, PosCreator, PosOpt};
