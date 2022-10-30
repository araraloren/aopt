pub(crate) mod forward;
pub(crate) mod process;
pub(crate) mod style;

pub use self::forward::Forward;
pub use self::style::Guess;
pub use self::style::GuessNOACfg;
pub use self::style::GuessOptCfg;
pub use self::style::NOAGuess;
pub use self::style::OptGuess;
pub use self::style::UserStyle;

pub(crate) use self::process::invoke_callback_opt;
pub(crate) use self::process::process_non_opt;
pub(crate) use self::process::process_opt;

use crate::args::Args;
use crate::ctx::Ctx;
use crate::ser::Services;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::Uid;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct CtxSaver {
    /// option uid
    pub uid: Uid,

    /// invoke context
    pub ctx: Ctx,
}

pub trait Policy {
    type Ret;
    type Set: Set;
    type Error: Into<Error>;

    fn parse(
        &mut self,
        args: Arc<Args>,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error>;
}
