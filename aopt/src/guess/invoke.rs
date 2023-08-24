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
use super::GuessOpt;
use super::GuessPolicy;
use super::MatchPolicy;
use super::MultiOpt;
use super::Process;
use super::SimpleMatRes;
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
    pub fn new(
        ctx: &'a mut Ctx,
        set: &'a mut Set,
        inv: &'a mut Inv,
        ser: &'a mut Ser,
        fail: &'a mut FailManager,
    ) -> Self {
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

    pub fn set_ctx(&mut self, ctx: &'a mut Ctx) -> &mut Self {
        self.ctx = ctx;
        self
    }

    pub fn set_optset(&mut self, set: &'a mut Set) -> &mut Self {
        self.set = set;
        self
    }

    pub fn set_inv(&mut self, inv: &'a mut Inv) -> &mut Self {
        self.inv = inv;
        self
    }

    pub fn set_ser(&mut self, ser: &'a mut Ser) -> &mut Self {
        self.ser = ser;
        self
    }

    pub fn set_fail(&mut self, fail: &'a mut FailManager) -> &mut Self {
        self.fail = fail;
        self
    }

    pub fn set_idx(&mut self, idx: usize) -> &mut Self {
        self.idx = idx;
        self
    }

    pub fn set_tot(&mut self, tot: usize) -> &mut Self {
        self.tot = tot;
        self
    }

    pub fn set_arg(&mut self, arg: Option<ARef<RawVal>>) -> &mut Self {
        self.arg = arg;
        self
    }

    pub fn set_name(&mut self, name: Option<Str>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_next(&mut self, next: Option<ARef<RawVal>>) -> &mut Self {
        self.next = next;
        self
    }

    pub fn with_ctx(mut self, ctx: &'a mut Ctx) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn with_set(mut self, set: &'a mut Set) -> Self {
        self.set = set;
        self
    }

    pub fn with_inv(mut self, inv: &'a mut Inv) -> Self {
        self.inv = inv;
        self
    }

    pub fn with_ser(mut self, ser: &'a mut Ser) -> Self {
        self.ser = ser;
        self
    }

    pub fn with_fail(mut self, fail: &'a mut FailManager) -> Self {
        self.fail = fail;
        self
    }

    pub fn with_idx(mut self, idx: usize) -> Self {
        self.idx = idx;
        self
    }

    pub fn with_tot(mut self, tot: usize) -> Self {
        self.tot = tot;
        self
    }

    pub fn with_arg(mut self, arg: Option<ARef<RawVal>>) -> Self {
        self.arg = arg;
        self
    }

    pub fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    pub fn with_next(mut self, next: Option<ARef<RawVal>>) -> Self {
        self.next = next;
        self
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

impl<'a, 'b, Set, Inv, Ser> InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    pub fn guess(&mut self, style: &UserStyle) -> Result<(bool, bool), Error> {
        // match style {
        //     UserStyle::Main => self.guess_wrapper::<MainStyle>(),
        //     UserStyle::Pos => self.guess_wrapper::<PosStyle>(),
        //     UserStyle::Cmd => self.guess_wrapper::<CmdStyle>(),
        //     UserStyle::EqualWithValue => self.guess_wrapper::<EqualWithValuStyle>(),
        //     UserStyle::Argument => self.guess_wrapper::<ArgumentStyle>(),
        //     UserStyle::EmbeddedValue => self.guess_wrapper::<EmbeddedValueStyle>(),
        //     UserStyle::EmbeddedValuePlus => self.guess_wrapper::<EmbeddedValuePlusStyle>(),
        //     UserStyle::CombinedOption => self.guess_wrapper::<CombinedOptionStyle>(),
        //     UserStyle::Boolean => self.guess_wrapper::<BooleanStyle>(),
        //     UserStyle::Flag => self.guess_wrapper::<FlagStyle>(),
        // }
        todo!()
    }
}

impl<'a, Set, Inv, Ser> GuessPolicy<ArgumentStyle> for InvokeGuess<'a, Set, Inv, Ser> {
    type All = SingleOpt<Set>;

    type First = FirstOpt<Set>;

    type Error = Error;

    fn guess_all(&mut self) -> Result<Option<Self::All>, Self::Error> {
        todo!()
    }

    fn guess_first(&mut self) -> Result<Option<Self::First>, Self::Error> {
        todo!()
    }
}

impl<'a, 'b, Set, Inv, Ser> Process<SingleOpt<Set>> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    // index of matched uid
    type Ret = Option<usize>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut SingleOpt<Set>) -> Result<bool, Self::Error> {
        let uids = self.set.keys();

        for uid in uids {
            // select all the option may match the `policy`
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

            // invoke the handler of `uid`
            let invoke_ret = self.inv.invoke_fb(uid, self.set, self.ser, self.ctx);

            // return first index if handler success
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

impl<'a, 'b, Set, Inv, Ser> Process<MultiOpt<SingleOpt<Set>, Set>>
    for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    // (index of matched policy, index of matched uid)
    type Ret = Vec<(usize, usize)>;

    type Error = Error;

    fn match_all(
        &mut self,
        policy: &mut MultiOpt<SingleOpt<Set>, Set>,
    ) -> Result<bool, Self::Error> {
        let uids = self.set.keys();
        let any_match = policy.any_match();

        for sub_policy in policy.sub_policys_mut() {
            // process all uids with each policy first
            for uid in uids.iter() {
                if !sub_policy.filter(*uid, self.set) {
                    if let Err(e) = sub_policy.r#match(*uid, self.set) {
                        if e.is_failure() {
                            self.fail.push(e);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            if any_match && sub_policy.matched() {
                break;
            }
        }
        Ok(policy.matched())
    }

    fn invoke_handler(
        &mut self,
        policy: &mut MultiOpt<SingleOpt<Set>, Set>,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ret = vec![];
        let any_match = policy.any_match();

        for (policy_idx, sub_policy) in policy.sub_policys_mut().iter_mut().enumerate() {
            let single = self.invoke_handler(sub_policy)?;

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
    // all the index of matched uids
    type Ret = Vec<usize>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut SingleNonOpt<Set>) -> Result<bool, Self::Error> {
        let uids = self.set.keys();

        // process all the uid
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

        for (index, uid) in policy.uids().iter().enumerate() {
            self.ctx
                .set_inner_ctx(Some(inner_ctx.clone().with_uid(*uid)));

            // invoke the handler of `uid`
            let invoke_ret = self.inv.invoke_fb(uid, self.set, self.ser, self.ctx);

            // add the index to return value
            if process_handler_ret(
                invoke_ret,
                |_| Ok(()),
                |e| {
                    self.fail.push(e);
                    Ok(())
                },
            )? {
                ret.push(index);
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
    // always 0 if exists
    type Ret = Option<usize>;

    type Error = Error;

    fn match_all(&mut self, policy: &mut FirstOpt<Set>) -> Result<bool, Self::Error> {
        let uids = self.set.keys();

        for uid in uids {
            // if any opt matched, skip
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

            // invoke the handler of first matched opt
            let invoke_ret = self.inv.invoke_fb(uid, self.set, self.ser, self.ctx);

            // return 0 if success
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

impl<'a, 'b, Set, Inv, Ser> Process<MultiOpt<FirstOpt<Set>, Set>> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    // index of matched policy
    // index of matched uid == 0
    type Ret = Vec<usize>;

    type Error = Error;

    fn match_all(
        &mut self,
        policy: &mut MultiOpt<FirstOpt<Set>, Set>,
    ) -> Result<bool, Self::Error> {
        let uids = self.set.keys();
        let any_match = policy.any_match();

        for sub_policy in policy.sub_policys_mut() {
            // process all uids with each policy first
            for uid in uids.iter() {
                if !sub_policy.matched() && !sub_policy.filter(*uid, self.set) {
                    if let Err(e) = sub_policy.r#match(*uid, self.set) {
                        if e.is_failure() {
                            self.fail.push(e);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            if any_match && sub_policy.matched() {
                break;
            }
        }
        Ok(policy.matched())
    }

    fn invoke_handler(
        &mut self,
        policy: &mut MultiOpt<FirstOpt<Set>, Set>,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ret = vec![];
        let any_match = policy.any_match();

        for (policy_idx, sub_policy) in policy.sub_policys_mut().iter_mut().enumerate() {
            let single = self.invoke_handler(sub_policy)?;

            // index is 0
            if let Some(_) = single {
                if any_match {
                    // any match, return current
                    return Ok(vec![policy_idx]);
                } else {
                    ret.push(policy_idx);
                }
            } else if !any_match {
                return Ok(vec![]);
            }
        }
        Ok(ret)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<MainStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = SimpleMatRes;

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Main;
        let name = self.name.clone();
        let args = self.ctx.args().clone();

        Ok(Some(
            SingleNonOpt::default()
                .with_idx(idx)
                .with_args(args)
                .with_name(name)
                .with_total(tot)
                .with_style(style)
                .reset_arg(),
        ))
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        let mut res = SimpleMatRes::default();

        if let Some(policy) = policy {
            if self.match_all(policy)? {
                for idx in self.invoke_handler(policy)? {
                    policy.apply(policy.uids()[idx], self.set)?;
                }
                res.matched = true;
            }
        }
        Ok(res)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<PosStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = SimpleMatRes;

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Pos;
        let name = self.name.clone();
        let args = self.ctx.args().clone();

        Ok(Some(
            SingleNonOpt::default()
                .with_idx(idx)
                .with_args(args)
                .with_name(name)
                .with_total(tot)
                .with_style(style)
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
    type Ret = SimpleMatRes;

    type Policy = Option<SingleNonOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Cmd;
        let name = self.name.clone();
        let args = self.ctx.args().clone();

        Ok(Some(
            SingleNonOpt::default()
                .with_idx(idx)
                .with_args(args)
                .with_name(name)
                .with_total(tot)
                .with_style(style)
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
    type Ret = SimpleMatRes;

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());

        let idx = self.idx;
        let tot = self.tot;
        let arg = self.arg.clone();
        let style = Style::Argument;
        let name = self.name.as_ref().unwrap().clone();

        Ok(self.arg.as_ref().map(|_| {
            SingleOpt::default()
                .with_idx(idx)
                .with_total(tot)
                .with_name(name)
                .with_arg(arg)
                .with_style(style)
        }))
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        let mut ret = SimpleMatRes::default();

        if let Some(policy) = policy {
            if self.match_all(policy)? {
                if let Some(idx) = self.invoke_handler(policy)? {
                    policy.apply(policy.uids()[idx], self.set)?;
                    ret.matched = true;
                    ret.consume = policy.is_consume();
                }
            }
        }
        Ok(ret)
    }
}

impl<'a, 'b, Set, Inv, Ser> GuessOpt<ArgumentStyle> for InvokeGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    type Ret = SimpleMatRes;

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());

        let idx = self.idx;
        let tot = self.tot;
        let arg = self.next.clone();
        let style = Style::Argument;
        let name = self.name.as_ref().unwrap().clone();

        Ok(if self.arg.is_none() && self.next.is_some() {
            Some(
                SingleOpt::default()
                    .with_idx(idx)
                    .with_total(tot)
                    .with_name(name)
                    .with_arg(arg)
                    .with_consume(true)
                    .with_style(style),
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
    type Ret = SimpleMatRes;

    type Policy = Option<SingleOpt<Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());

        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Argument;

        if self.arg.is_none() {
            // strip the prefix before generate
            let option = self.name.as_ref().unwrap().as_str();
            let validator = &self.set;
            let splited = validator.split(option).map_err(Into::into)?;

            // make sure we using `chars.count`, not len()
            // make sure the name length >= 2
            // only check first letter `--v42` ==> `--v 42`
            if let Some((idx, _)) = splited.1.char_indices().nth(1) {
                let (name, arg) = splited.1.split_at(idx);
                let arg = Some(RawVal::from(arg).into());
                let name = name.into();

                return Ok(Some(
                    SingleOpt::default()
                        .with_idx(idx)
                        .with_total(tot)
                        .with_name(name)
                        .with_arg(arg)
                        .with_style(style),
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
    type Ret = SimpleMatRes;

    type Policy = Option<MultiOpt<SingleOpt<Set>, Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());

        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Argument;

        if self.arg.is_none() {
            // strip the prefix before generate
            let option = self.name.as_ref().unwrap().as_str();
            let validator = &self.set;
            let splited = validator.split(option).map_err(Into::into)?;
            let char_indices = splited.1.char_indices().skip(2);
            let mut policy = MultiOpt::default().with_any_match(true);

            // make sure we using `chars.count`, not len()
            // check the name start 3th letter
            // for `--opt42` check the option like `--op t42`, `--opt 42`, `--opt4 2`
            for (idx, _) in char_indices {
                let (name, arg) = splited.1.split_at(idx);
                let arg = Some(RawVal::from(arg).into());
                let name = name.into();

                policy.add_sub_policy(
                    SingleOpt::default()
                        .with_idx(idx)
                        .with_total(tot)
                        .with_name(name)
                        .with_arg(arg)
                        .with_consume(false)
                        .with_style(style),
                );
            }
            Ok(Some(policy))
        } else {
            Ok(None)
        }
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        let mut ret = SimpleMatRes::default();

        if let Some(policy) = policy {
            if !policy.is_empty() {
                if self.match_all(policy)? {
                    let invoke_rets = self.invoke_handler(policy)?;

                    if !invoke_rets.is_empty() {
                        ret.matched = true;
                        for (policy_idx, uid_idx) in invoke_rets {
                            let sub_policy = &mut policy.sub_policys_mut()[policy_idx];

                            sub_policy.apply(sub_policy.uids()[uid_idx], self.set)?;
                            ret.consume = ret.consume || sub_policy.is_consume();
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
    type Ret = SimpleMatRes;

    type Policy = Option<MultiOpt<SingleOpt<Set>, Set>>;

    type Error = Error;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());

        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Boolean;
        let arg = Some(ARef::new(RawVal::from(BOOL_TRUE)));

        if self.arg.is_none() {
            // strip the prefix before generate
            let option = self.name.as_ref().unwrap().as_str();
            let validator = &self.set;
            let splited = validator.split(option).map_err(Into::into)?;

            if splited.1.chars().count() > 1 {
                let mut policy = MultiOpt::default().with_any_match(false);

                for ch in splited.1.chars() {
                    policy.add_sub_policy(
                        SingleOpt::default()
                            .with_idx(idx)
                            .with_total(tot)
                            .with_name(format!("{}{}", splited.0, ch).into())
                            .with_arg(arg.clone())
                            .with_style(style),
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
    type Ret = SimpleMatRes;

    type Error = Error;

    type Policy = Option<SingleOpt<Set>>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());

        let idx = self.idx;
        let tot = self.tot;
        let arg = Some(ARef::new(RawVal::from(BOOL_TRUE)));
        let style = Style::Boolean;
        let name = self.name.as_ref().unwrap().clone();

        if self.arg.is_none() {
            Ok(Some(
                SingleOpt::default()
                    .with_idx(idx)
                    .with_total(tot)
                    .with_name(name)
                    .with_arg(arg)
                    .with_style(style),
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
    type Ret = SimpleMatRes;

    type Error = Error;

    type Policy = Option<SingleOpt<Set>>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error> {
        debug_assert!(self.name.is_some());

        let idx = self.idx;
        let tot = self.tot;
        let arg = None;
        let style = Style::Flag;
        let name = self.name.as_ref().unwrap().clone();

        Ok(if self.arg.is_none() {
            Some(
                SingleOpt::default()
                    .with_idx(idx)
                    .with_total(tot)
                    .with_name(name)
                    .with_arg(arg)
                    .with_style(style),
            )
        } else {
            None
        })
    }

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error> {
        GuessOpt::<EqualWithValuStyle>::guess_opt(self, policy)
    }
}
