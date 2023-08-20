use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::HandlerCollection;
use crate::ctx::InnerCtx;
use crate::parser::UserStyle;
use crate::set::OptValidator;
use crate::Error;
use crate::Uid;

use super::style::*;
use super::Guess;
use super::GuessOpt;
use super::InvokeGuess;
use super::MultiOpt;
use super::Process;
use super::SingleNonOpt;
use super::SingleOpt;

#[derive(Debug)]
pub struct DelayGuess<'a, Set, Inv, Ser>(pub InvokeGuess<'a, Set, Inv, Ser>);

impl<'a, 'b, Set, Inv, Ser> Deref for DelayGuess<'a, Set, Inv, Ser> {
    type Target = InvokeGuess<'a, Set, Inv, Ser>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'b, Set, Inv, Ser> DerefMut for DelayGuess<'a, Set, Inv, Ser> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, 'b, Set, Inv, Ser> DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    fn guess_wrapper<T>(
        &mut self,
    ) -> Result<<Self as GuessOpt<T>>::Ret, <Self as GuessOpt<T>>::Error>
    where
        Self: GuessOpt<T>,
    {
        let mut policy = GuessOpt::<T>::guess_policy(self)?;

        GuessOpt::<T>::guess_opt(self, &mut policy)
    }
}

#[derive(Debug)]
pub struct InnerCtxSaver {
    pub uid: Vec<Uid>,

    pub ctx: InnerCtx,
}

impl<'a, 'b, Set, Inv, Ser> Guess for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Sty = UserStyle;

    type Ret = Option<(bool, Vec<InnerCtxSaver>)>;

    type Error = Error;

    fn guess(&mut self, style: &Self::Sty) -> Result<Self::Ret, Self::Error> {
        match style {
            UserStyle::Main => {
                let ret = self.guess_wrapper::<MainStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
            UserStyle::Pos => {
                let ret = self.guess_wrapper::<PosStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
            UserStyle::Cmd => {
                let ret = self.guess_wrapper::<CmdStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
            UserStyle::EqualWithValue => {
                let ret = self.guess_wrapper::<EqualWithValuStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
            UserStyle::Argument => {
                let ret = self.guess_wrapper::<ArgumentStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
            UserStyle::EmbeddedValue => {
                let ret = self.guess_wrapper::<EmbeddedValueStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
            UserStyle::EmbeddedValuePlus => self.guess_wrapper::<EmbeddedValuePlusStyle>(),
            UserStyle::CombinedOption => self.guess_wrapper::<CombinedOptionStyle>(),
            UserStyle::Boolean => {
                let ret = self.guess_wrapper::<BooleanStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
            UserStyle::Flag => {
                let ret = self.guess_wrapper::<FlagStyle>()?;
                Ok(ret.map(|v| (false, vec![v])))
            }
        }
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<SingleOpt<Set>> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut SingleOpt<Set>) -> Result<bool, Self::Error> {
        Process::<SingleOpt<Set>>::match_all(&mut self.0, policy)
    }

    fn invoke_handler(&mut self, policy: &mut SingleOpt<Set>) -> Result<Self::Ret, Self::Error> {
        if policy.uids().is_empty() {
            Ok(None)
        } else {
            Ok(Some(InnerCtxSaver {
                uid: policy.uids().to_vec(),
                ctx: InnerCtx::default()
                    .with_idx(policy.idx())
                    .with_total(policy.total())
                    .with_name(policy.name().cloned())
                    .with_arg(policy.clone_arg())
                    .with_style(policy.style()),
            }))
        }
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<MultiOpt<Set>> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<(bool, Vec<InnerCtxSaver>)>;

    type Error = Error;

    fn match_all(&mut self, multi_policy: &mut MultiOpt<Set>) -> Result<bool, Self::Error> {
        Process::<MultiOpt<Set>>::match_all(&mut self.0, multi_policy)
    }

    fn invoke_handler(
        &mut self,
        multi_policy: &mut MultiOpt<Set>,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ret = vec![];
        let any_match = multi_policy.any_match();

        for (_, policy) in multi_policy.sub_policys_mut().iter_mut().enumerate() {
            let single = self.invoke_handler(policy)?;

            if let Some(ctx) = single {
                ret.push(ctx);
            }
        }
        if ret.is_empty() {
            Ok(None)
        } else {
            Ok(Some((any_match, ret)))
        }
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<SingleNonOpt<Set>> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut SingleNonOpt<Set>) -> Result<bool, Self::Error> {
        Process::<SingleNonOpt<Set>>::match_all(&mut self.0, policy)
    }

    fn invoke_handler(&mut self, policy: &mut SingleNonOpt<Set>) -> Result<Self::Ret, Self::Error> {
        if policy.uids().is_empty() {
            Ok(None)
        } else {
            Ok(Some(InnerCtxSaver {
                uid: policy.uids().to_vec(),
                ctx: InnerCtx::default()
                    .with_idx(policy.idx())
                    .with_total(policy.total())
                    .with_name(policy.name().cloned())
                    .with_arg(policy.clone_arg())
                    .with_style(policy.style()),
            }))
        }
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<MainStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<MainStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        if let Some(policy) = policy {
            if self.match_all(policy)? {
                return self.invoke_handler(policy);
            }
        }
        Ok(None)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<PosStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<PosStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<MainStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<CmdStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<CmdStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<MainStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<EqualWithValuStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        if let Some(policy) = policy {
            if self.match_all(policy)? {
                return self.invoke_handler(policy);
            }
        }
        Ok(None)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<ArgumentStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<ArgumentStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<EmbeddedValueStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<EmbeddedValueStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<EmbeddedValuePlusStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<(bool, Vec<InnerCtxSaver>)>;

    type Policy = Option<MultiOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<EmbeddedValuePlusStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        if let Some(policy) = policy {
            if !policy.is_empty() {
                if self.match_all(policy)? {
                    return self.invoke_handler(policy);
                }
            }
        }
        Ok(None)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<CombinedOptionStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<(bool, Vec<InnerCtxSaver>)>;

    type Policy = Option<MultiOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<CombinedOptionStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EmbeddedValuePlusStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<BooleanStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Error = Error;

    type Policy = Option<SingleOpt<Set>>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<BooleanStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<FlagStyle> for DelayGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<InnerCtxSaver>;

    type Error = Error;

    type Policy = Option<SingleOpt<Set>>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        GuessOpt::<FlagStyle>::guess_policy(&mut self.0)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}
