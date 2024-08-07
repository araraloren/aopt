mod invoke;
mod multi;
mod noa;
mod single;
pub mod style;

use crate::args::Args;
use crate::ctx::InnerCtx;
use crate::opt::Style;
use crate::ARef;
use crate::AStr;
use crate::Error;
use crate::RawVal;
use crate::Uid;

pub use self::invoke::InvokeGuess;
pub use self::multi::MultiOpt;
pub use self::noa::SingleNonOpt;
pub use self::single::SingleOpt;

#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleMatRet {
    pub matched: bool,

    pub consume: bool,
}

impl SimpleMatRet {
    pub fn new(matched: bool, consume: bool) -> Self {
        Self { matched, consume }
    }
}

//
// argument boolean/flag embedded equalwithvalue - generate one guess
//     - invoke
//         - first
//             - match first opt
//             - invoke the handler of first opt
//             - set first opt matched if handler return Ok(Some(_))
//         - all
//             - match all the opt
//             - invoke the handler of all matched opt
//             - set opt matched and return if any handler return Ok(Some(_))
//    - delay
//         - first
//             - match first opt
//             - return the inner ctx
//         - all
//             - match all the opt
//             - return the inner ctxs
//
// embeddedplus combined - generate multiple guess
//     - invoke
//         - first
//             - match first opt
//             - invoke the handler of first opt
//             - set first opt matched if handler return Ok(Some(_))
//         - all
//             - match all the opt
//             - invoke the handler of all matched opt
//             - set opt matched and return if any handler return Ok(Some(_))
//    - delay
//         - first
//             - match first opt
//             - return the inner ctx
//         - all
//             - match all the opt
//             - return the inner ctxs
// main pos cmd - generate one guess
//     - invoke
//         - match all the opt
//         - invoke the handler of all matched opt
//         - set all the opt matched if handler return Ok(Some(_))
//     - delay mode
//         not support
//
pub trait GuessPolicy<Sty, Policy> {
    type Error: Into<Error>;

    fn guess_policy(&mut self) -> Result<Option<Policy>, Self::Error>;
}

#[derive(Debug, Clone, Default)]
pub struct PolicyInnerCtx {
    pub uids: Vec<Uid>,

    pub inner_ctx: InnerCtx,
}

#[derive(Debug, Clone, Default)]
pub struct InnerCtxSaver {
    pub any_match: bool,

    pub consume: bool,

    pub policy_ctx: Vec<PolicyInnerCtx>,
}

impl InnerCtxSaver {
    pub fn with_any_match(mut self, any_match: bool) -> Self {
        self.any_match = any_match;
        self
    }

    pub fn with_consume(mut self, consume: bool) -> Self {
        self.consume = consume;
        self
    }

    pub fn with_policy_ctx(mut self, policy_ctx: Vec<PolicyInnerCtx>) -> Self {
        self.policy_ctx = policy_ctx;
        self
    }
}

pub trait MatchPolicy {
    type Set;
    type Ret;
    type Error: Into<Error>;

    fn reset(&mut self) -> &mut Self;

    fn matched(&self) -> bool;

    fn undo(&mut self, uid: Uid, set: &mut Self::Set) -> Result<(), Self::Error>;

    fn apply(&mut self, uid: Uid, set: &mut Self::Set) -> Result<(), Self::Error>;

    fn filter(&mut self, uid: Uid, set: &mut Self::Set) -> bool;

    fn r#match(
        &mut self,
        uid: Uid,
        set: &mut Self::Set,
        overload: bool,
        consume: bool,
    ) -> Result<Self::Ret, Self::Error>;
}

pub trait PolicyConfig {
    fn idx(&self) -> usize;

    fn tot(&self) -> usize;

    fn name(&self) -> Option<&AStr>;

    fn style(&self) -> Style;

    fn arg(&self) -> Option<RawVal>;

    fn uids(&self) -> &[Uid];

    fn collect_ctx(&self) -> Option<PolicyInnerCtx>;
}

pub trait PolicyBuild {
    fn with_name(self, name: Option<AStr>) -> Self;

    fn with_style(self, style: Style) -> Self;

    fn with_idx(self, index: usize) -> Self;

    fn with_tot(self, total: usize) -> Self;

    fn with_arg(self, argument: Option<RawVal>) -> Self;

    fn with_args(self, args: ARef<Args>) -> Self;
}

/// Process the return value of handler:
/// call the callback `when_ret` and return the return value of handler if `Ok`;
/// ignore failure and call the callback `when_fail` on the failure if `Err`
/// or the return the `Err`.
pub fn process_handler_ret(
    ret: Result<bool, Error>,
    mut when_ret: impl FnMut(bool) -> Result<(), Error>,
    mut when_fail: impl FnMut(Error) -> Result<(), Error>,
) -> Result<bool, Error> {
    match ret {
        Ok(ret) => {
            (when_ret)(ret)?;
            Ok(ret)
        }
        Err(e) => {
            if e.is_failure() {
                (when_fail)(e)?;
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}
