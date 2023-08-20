mod delay;
mod invoke;
mod multi;
mod noa;
mod single;
mod style;

use crate::Error;
use crate::Uid;

pub use self::invoke::InvokeGuess;
pub use self::multi::MultiOpt;
pub use self::noa::SingleNonOpt;
pub use self::single::SingleOpt;

pub trait Guess {
    type Sty;
    type Ret;
    type Error: Into<Error>;

    fn guess(&mut self, style: &Self::Sty) -> Result<Self::Ret, Self::Error>;
}

pub trait GuessOpt<T> {
    type Ret;
    type Policy;
    type Error: Into<Error>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error>;

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error>;
}

pub trait Process<Policy> {
    type Ret;
    type Error: Into<Error>;

    fn process(&mut self, policy: &mut Policy) -> Result<bool, Self::Error>;

    fn invoke_handler(&mut self, policy: &mut Policy) -> Result<Self::Ret, Self::Error>;
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

    fn r#match(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Self::Ret, Self::Error>;
}

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
