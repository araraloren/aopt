use crate::ctx::Ctx;
use crate::ctx::HandlerCollection;
use crate::ctx::InnerCtx;
use crate::opt::Style;
use crate::opt::BOOL_TRUE;
use crate::parser::FailManager;
use crate::parser::UserStyle;
use crate::set::OptValidator;
use crate::ARef;
use crate::Error;
use crate::RawVal;
use crate::Str;

use super::process_handler_ret;
use super::style::*;
use super::FirstOpt;
use super::Guess;
use super::GuessOpt;
use super::MatchPolicy;
use super::MultiOpt;
use super::Process;
use super::SingleNonOpt;
use super::SingleOpt;

#[derive(Debug)]
pub struct InvokeGuess<'a, Set, Inv, Ser> {
    pub idx: usize,

    pub tot: usize,

    pub arg: Option<ARef<RawVal>>,

    pub name: Option<Str>,

    pub next: Option<ARef<RawVal>>,

    pub ctx: &'a mut Ctx,

    pub set: &'a mut Set,

    pub inv: &'a mut Inv,

    pub ser: &'a mut Ser,

    pub fail: &'a mut FailManager,
}

impl<'a, Set, Inv, Ser> InvokeGuess<'a, Set, Inv, Ser> {
    pub fn new(ctx: &'a mut Ctx, set: &'a mut Set, inv: &'a mut Inv, ser: &'a mut Ser, fail: &'a mut FailManager) -> Self {
        Self {
            idx: 0,
            tot: 0,
            arg: None,
            name: None,
            next: None,
            ctx,
            set,
            inv,
            ser,
            fail,
        }
    }
}

impl<'a, 'b, Set, Inv, Ser> InvokeGuess<'a, Set, Inv, Ser>
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

impl<'a, 'b, Set, Inv, Ser> Guess for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Sty = UserStyle;

    type Ret = (bool, bool);

    type Error = Error;

    fn guess(&mut self, style: &Self::Sty) -> Result<Self::Ret, Self::Error> {
        match style {
            UserStyle::Main => self.guess_wrapper::<MainStyle>(),
            UserStyle::Pos => self.guess_wrapper::<PosStyle>(),
            UserStyle::Cmd => self.guess_wrapper::<CmdStyle>(),
            UserStyle::EqualWithValue => self.guess_wrapper::<EqualWithValuStyle>(),
            UserStyle::Argument => self.guess_wrapper::<ArgumentStyle>(),
            UserStyle::EmbeddedValue => self.guess_wrapper::<EmbeddedValueStyle>(),
            UserStyle::EmbeddedValuePlus => self.guess_wrapper::<EmbeddedValuePlusStyle>(),
            UserStyle::CombinedOption => self.guess_wrapper::<CombinedOptionStyle>(),
            UserStyle::Boolean => self.guess_wrapper::<BooleanStyle>(),
            UserStyle::Flag => self.guess_wrapper::<FlagStyle>(),
        }
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<SingleOpt<Set>> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<usize>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut SingleOpt<Set>) -> Result<bool, Self::Error> {
        let uids = self.set.keys();

        for uid in uids {
            if !policy.filter(uid, self.set) {
                if let Err(e) = policy.r#match(uid, self.set) {
                    if e.is_failure() {
                        self.fail.push(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(policy.matched())
    }

    fn invoke_handler(&mut self, policy: &mut SingleOpt<Set>) -> Result<Self::Ret, Self::Error> {
        let inner_ctx = InnerCtx::default()
            .with_idx(policy.idx())
            .with_total(policy.total())
            .with_name(policy.name().cloned())
            .with_arg(policy.clone_arg())
            .with_style(policy.style());

        for (idx, uid) in policy.uids().iter().enumerate() {
            self.ctx
                .set_inner_ctx(Some(inner_ctx.clone().with_uid(*uid)));

            let invoke_ret = self.inv.invoke_fb(uid, self.set, self.ser, self.ctx);

            if process_handler_ret(
                invoke_ret,
                |_| Ok(()),
                |e| {
                    self.fail.push(e);
                    Ok(())
                },
            )? {
                return Ok(Some(idx));
            }
        }
        Ok(None)
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<MultiOpt<Set>> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Vec<(usize, usize)>;

    type Error = Error;

    fn match_all(&mut self, multi_policy: &mut MultiOpt<Set>) -> Result<bool, Self::Error> {
        let uids = self.set.keys();
        let any_match = multi_policy.any_match();

        for policy in multi_policy.sub_policys_mut() {
            for uid in uids.iter() {
                if !policy.filter(*uid, self.set) {
                    if let Err(e) = policy.r#match(*uid, self.set) {
                        if e.is_failure() {
                            self.fail.push(e);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            if any_match && policy.matched() {
                break;
            }
        }
        Ok(multi_policy.matched())
    }

    fn invoke_handler(
        &mut self,
        multi_policy: &mut MultiOpt<Set>,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ret = vec![];
        let any_match = multi_policy.any_match();

        for (policy_idx, policy) in multi_policy.sub_policys_mut().iter_mut().enumerate() {
            let single = self.invoke_handler(policy)?;

            if let Some(idx) = single {
                if any_match {
                    // any match, return current
                    return Ok(vec![(policy_idx, idx)]);
                } else {
                    ret.push((policy_idx, idx));
                }
            } else if !any_match {
                return Ok(vec![]);
            }
        }
        Ok(ret)
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<SingleNonOpt<Set>> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Vec<usize>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut SingleNonOpt<Set>) -> Result<bool, Self::Error> {
        let uids = self.set.keys();

        for uid in uids {
            if !policy.filter(uid, self.set) {
                if let Err(e) = policy.r#match(uid, self.set) {
                    if e.is_failure() {
                        self.fail.push(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(policy.matched())
    }

    fn invoke_handler(&mut self, policy: &mut SingleNonOpt<Set>) -> Result<Self::Ret, Self::Error> {
        let mut ret = vec![];
        let inner_ctx = InnerCtx::default()
            .with_idx(policy.idx())
            .with_total(policy.total())
            .with_name(policy.name().cloned())
            .with_arg(policy.clone_arg())
            .with_style(policy.style());

        for (idx, uid) in policy.uids().iter().enumerate() {
            self.ctx
                .set_inner_ctx(Some(inner_ctx.clone().with_uid(*uid)));

            let invoke_ret = self.inv.invoke_fb(uid, self.set, self.ser, self.ctx);

            if process_handler_ret(
                invoke_ret,
                |_| Ok(()),
                |e| {
                    self.fail.push(e);
                    Ok(())
                },
            )? {
                ret.push(idx);
            }
        }
        Ok(ret)
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<FirstOpt<Set>> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = Option<usize>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut FirstOpt<Set>) -> Result<bool, Self::Error> {
        let uids = self.set.keys();

        for uid in uids {
            if !policy.matched() && !policy.filter(uid, self.set) {
                if let Err(e) = policy.r#match(uid, self.set) {
                    if e.is_failure() {
                        self.fail.push(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(policy.matched())
    }

    fn invoke_handler(&mut self, policy: &mut FirstOpt<Set>) -> Result<Self::Ret, Self::Error> {
        let inner_ctx = InnerCtx::default()
            .with_idx(policy.idx())
            .with_total(policy.total())
            .with_name(policy.name().cloned())
            .with_arg(policy.clone_arg())
            .with_style(policy.style());

        if let Some(uid) = policy.uid() {
            self.ctx
                .set_inner_ctx(Some(inner_ctx.clone().with_uid(*uid)));

            let invoke_ret = self.inv.invoke_fb(uid, self.set, self.ser, self.ctx);

            if process_handler_ret(
                invoke_ret,
                |_| Ok(()),
                |e| {
                    self.fail.push(e);
                    Ok(())
                },
            )? {
                return Ok(Some(0));
            }
        }
        Ok(None)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<MainStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        let args = self.ctx.args();

        Ok(Some(
            SingleNonOpt::default()
                .with_idx(self.idx)
                .with_total(self.tot)
                .with_args(args.clone())
                .with_style(Style::Main)
                .with_name(self.name.clone())
                .reset_arg(),
        ))
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        if let Some(policy) = policy {
            if self.match_all(policy)? {
                for idx in self.invoke_handler(policy)? {
                    policy.apply(policy.uids()[idx], self.set)?;
                }
                return Ok((true, false));
            }
        }
        Ok((false, false))
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<PosStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        let args = self.ctx.args();

        Ok(Some(
            SingleNonOpt::default()
                .with_idx(self.idx)
                .with_total(self.tot)
                .with_args(args.clone())
                .with_style(Style::Pos)
                .with_name(self.name.clone())
                .reset_arg(),
        ))
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<MainStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<CmdStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        let args = self.ctx.args();

        Ok(Some(
            SingleNonOpt::default()
                .with_idx(self.idx)
                .with_total(self.tot)
                .with_args(args.clone())
                .with_style(Style::Cmd)
                .with_name(self.name.clone())
                .with_arg(Some(RawVal::from(BOOL_TRUE).into())),
        ))
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<MainStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<EqualWithValuStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());
        Ok(self.arg.as_ref().map(|_| {
            SingleOpt::default()
                .with_idx(self.idx)
                .with_total(self.tot)
                .with_name(self.name.as_ref().unwrap().clone())
                .with_arg(self.arg.clone())
                .with_consume(false)
                .with_style(Style::Argument)
        }))
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        if let Some(policy) = policy {
            if self.match_all(policy)? {
                if let Some(idx) = self.invoke_handler(policy)? {
                    policy.apply(policy.uids()[idx], self.set)?;
                    return Ok((true, policy.is_consume()));
                }
            }
        }
        Ok((false, false))
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<ArgumentStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());
        Ok(if self.arg.is_none() && self.next.is_some() {
            Some(
                SingleOpt::default()
                    .with_idx(self.idx)
                    .with_total(self.tot)
                    .with_name(self.name.as_ref().unwrap().clone())
                    .with_arg(self.next.clone())
                    .with_consume(true)
                    .with_style(Style::Argument),
            )
        } else {
            None
        })
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<EmbeddedValueStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());
        if self.arg.is_none() {
            // strip the prefix before generate
            let name = self.name.as_ref().unwrap().as_str();
            let opt_validator = &self.set;
            let splited = opt_validator.split(name).map_err(Into::into)?;
            let prefix_len = splited.0.len();

            // make sure we using `chars.count`, not len()
            // make sure the name length >= 2
            // only check first letter `--v42` ==> `--v 42`
            if let Some((idx, _)) = splited.1.char_indices().nth(1) {
                let name_value = name.split_at(prefix_len + idx);

                return Ok(Some(
                    SingleOpt::default()
                        .with_idx(self.idx)
                        .with_total(self.tot)
                        .with_name(name_value.0.into())
                        .with_arg(Some(RawVal::from(name_value.1).into()))
                        .with_consume(false)
                        .with_style(Style::Argument),
                ));
            }
        }
        Ok(None)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<EmbeddedValuePlusStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<MultiOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());
        if self.arg.is_none() {
            // strip the prefix before generate
            let name = self.name.as_ref().unwrap().as_str();
            let opt_validator = &self.set;
            let splited = opt_validator.split(name).map_err(Into::into)?;
            let prefix_len = splited.0.len();
            let char_indices = splited.1.char_indices().skip(2);
            let mut policy = MultiOpt::default().with_any_match(true);

            // make sure we using `chars.count`, not len()
            // check the name start 3th letter
            // for `--opt42` check the option like `--op t42`, `--opt 42`, `--opt4 2`
            for (i, _) in char_indices {
                let name_value = name.split_at(prefix_len + i);

                policy.add_sub_policy(
                    SingleOpt::default()
                        .with_idx(self.idx)
                        .with_total(self.tot)
                        .with_name(name_value.0.into())
                        .with_arg(Some(RawVal::from(name_value.1).into()))
                        .with_consume(false)
                        .with_style(Style::Argument),
                );
            }
            Ok(Some(policy))
        } else {
            Ok(None)
        }
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        let mut ret = (false, false);
        if let Some(policy) = policy {
            if !policy.is_empty() {
                if self.match_all(policy)? {
                    let invoke_rets = self.invoke_handler(policy)?;

                    if !invoke_rets.is_empty() {
                        ret.0 = true;
                        for (policy_idx, idx) in invoke_rets {
                            let sub_policy = &mut policy.sub_policys_mut()[policy_idx];

                            sub_policy.apply(sub_policy.uids()[idx], self.set)?;
                            ret.1 = ret.1 || sub_policy.is_consume();
                        }
                    }
                }
            }
        }
        Ok(ret)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<CombinedOptionStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Policy = Option<MultiOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());
        if self.arg.is_none() {
            // strip the prefix before generate
            let name = self.name.as_ref().unwrap().as_str();
            let opt_validator = &self.set;
            let splited = opt_validator.split(name).map_err(Into::into)?;

            if splited.1.chars().count() > 1 {
                let mut policy = MultiOpt::default().with_any_match(false);

                for ch in splited.1.chars() {
                    policy.add_sub_policy(
                        SingleOpt::default()
                            .with_idx(self.idx)
                            .with_total(self.tot)
                            .with_name(format!("{}{}", splited.0, ch).into())
                            .with_arg(Some(ARef::new(RawVal::from(BOOL_TRUE))))
                            .with_consume(false)
                            .with_style(Style::Combined),
                    );
                }
                return Ok(Some(policy));
            }
        }
        Ok(None)
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EmbeddedValuePlusStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<BooleanStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Error = Error;

    type Policy = Option<SingleOpt<Set>>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());
        if self.arg.is_none() {
            Ok(Some(
                SingleOpt::default()
                    .with_idx(self.idx)
                    .with_total(self.tot)
                    .with_name(self.name.as_ref().unwrap().clone())
                    .with_arg(Some(ARef::new(RawVal::from(BOOL_TRUE))))
                    .with_consume(false)
                    .with_style(Style::Boolean),
            ))
        } else {
            Ok(None)
        }
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<FlagStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = (bool, bool);

    type Error = Error;

    type Policy = Option<SingleOpt<Set>>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());
        Ok(if self.arg.is_none() {
            Some(
                SingleOpt::default()
                    .with_idx(self.idx)
                    .with_total(self.tot)
                    .with_name(self.name.as_ref().unwrap().clone())
                    .with_arg(None)
                    .with_consume(false)
                    .with_style(Style::Flag),
            )
        } else {
            None
        })
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}
