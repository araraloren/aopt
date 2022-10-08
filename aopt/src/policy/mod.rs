pub(crate) mod common;
pub(crate) mod guess_style;
pub(crate) mod policy_delay;
pub(crate) mod policy_forward;
pub(crate) mod policy_pre;

pub use self::common::process_non_opt;
pub use self::common::process_opt;
pub use self::guess_style::Guess;
pub use self::guess_style::GuessNOACfg;
pub use self::guess_style::GuessOptCfg;
pub use self::guess_style::NOAGuess;
pub use self::guess_style::OptGuess;
pub use self::guess_style::UserStyle;
pub use self::policy_delay::DelayPolicy;
pub use self::policy_forward::ForwardPolicy;
pub use self::policy_pre::PrePolicy;

use crate::arg::Args;
use crate::ctx::Ctx;
use crate::ser::Services;
use crate::set::Set;
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

// todo ! change the Ret to Value; Add Ret for return value;
pub trait Policy {
    type Ret;
    type Value;
    type Set: Set;
    type Error: Into<Error>;

    fn parse(
        &mut self,
        args: Args,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error>;
}
