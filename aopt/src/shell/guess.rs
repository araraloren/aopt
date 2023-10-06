use crate::ctx::Ctx;
use crate::ctx::HandlerCollection;
use crate::opt::Style;
use crate::opt::BOOL_TRUE;
use crate::parser::UserStyle;
use crate::set::OptValidator;
use crate::ARef;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

use crate::guess::style::*;
use crate::guess::GuessPolicy;
use crate::guess::MatchPolicy;
use crate::guess::MultiOpt;
use crate::guess::PolicyBuild;
use crate::guess::PolicyConfig;
use crate::guess::SimpleMatRet;
use crate::guess::SingleNonOpt;
use crate::guess::SingleOpt;

#[derive(Debug, Clone, Copy)]
pub struct CompleteRet {
    pub matched: bool,

    pub consume: bool,

    pub need_val_opt: Option<Uid>,
}

#[derive(Debug)]
pub struct CompleteGuess<'a, Set, Inv, Ser> {
    pub idx: usize,

    pub tot: usize,

    pub arg: Option<ARef<RawVal>>,

    pub name: Option<Str>,

    pub next: Option<ARef<RawVal>>,

    pub ctx: &'a mut Ctx,

    pub set: &'a mut Set,

    pub inv: &'a mut Inv,

    pub ser: &'a mut Ser,
}

impl<'a, Set, Inv, Ser> CompleteGuess<'a, Set, Inv, Ser> {
    pub fn new(ctx: &'a mut Ctx, set: &'a mut Set, inv: &'a mut Inv, ser: &'a mut Ser) -> Self {
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

impl<'a, 'b, Set, Inv, Ser> CompleteGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    pub fn guess_complete(&mut self, style: &UserStyle) -> Result<Option<SimpleMatRet>, Error> {
        let mut matched = false;
        let mut consume = false;

        match style {
            UserStyle::Main => {
                if let Some(mut policy) =
                    GuessPolicy::<MainStyle, SingleNonOpt<Set>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, true)?;
                    }
                }
            }
            UserStyle::Pos => {
                if let Some(mut policy) =
                    GuessPolicy::<PosStyle, SingleNonOpt<Set>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, true)?;
                    }
                }
            }
            UserStyle::Cmd => {
                if let Some(mut policy) =
                    GuessPolicy::<CmdStyle, SingleNonOpt<Set>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, true)?;
                    }
                }
            }
            UserStyle::EqualWithValue => {
                if let Some(mut policy) =
                    GuessPolicy::<EqualWithValuStyle, SingleOpt<Set>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::Argument => {
                if let Some(mut policy) =
                    GuessPolicy::<ArgumentStyle, SingleOpt<Set>>::guess_policy(self)?
                {
                    consume = true;
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::EmbeddedValue => {
                if let Some(mut policy) =
                    GuessPolicy::<EmbeddedValueStyle, SingleOpt<Set>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::EmbeddedValuePlus => {
                if let Some(mut policy) = GuessPolicy::<
                    EmbeddedValuePlusStyle,
                    MultiOpt<SingleOpt<Set>, Set>,
                >::guess_policy(self)?
                {
                    if self.match_multi(&mut policy, consume)? {
                        matched = self.apply_multi(&mut policy, false)?;
                    }
                }
            }
            UserStyle::CombinedOption => {
                if let Some(mut policy) = GuessPolicy::<
                    CombinedOptionStyle,
                    MultiOpt<SingleOpt<Set>, Set>,
                >::guess_policy(self)?
                {
                    if self.match_multi(&mut policy, consume)? {
                        matched = self.apply_multi(&mut policy, false)?;
                    }
                }
            }
            UserStyle::Boolean => {
                if let Some(mut policy) =
                    GuessPolicy::<BooleanStyle, SingleOpt<Set>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::Flag => {
                if let Some(mut policy) =
                    GuessPolicy::<FlagStyle, SingleOpt<Set>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
        }
        Ok(Some(SimpleMatRet::new(matched, consume)))
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<EqualWithValuStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        if self.arg.is_some() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.tot)
                        .with_name(Some(name.clone()))
                        .with_arg(self.arg.clone())
                        .with_style(Style::Argument),
                ));
            }
        }
        Ok(None)
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<ArgumentStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        if self.arg.is_none() && self.next.is_some() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.tot)
                        .with_name(Some(name.clone()))
                        .with_arg(self.next.clone())
                        .with_style(Style::Argument),
                ));
            }
        }
        Ok(None)
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<EmbeddedValueStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    Set: OptValidator,
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Argument;

        if self.arg.is_none() {
            if let Some(name) = &self.name {
                // strip the prefix before generate
                let validator = &self.set;
                let splited = validator.split(name.as_str()).map_err(Into::into)?;
                let prefix_len = splited.0.len();

                // make sure we using `chars.count`, not len()
                // make sure the name length >= 2
                // only check first letter `--v42` ==> `--v 42`
                if let Some((char_idx, _)) = splited.1.char_indices().nth(1) {
                    let (name, arg) = name.split_at(prefix_len + char_idx);
                    let arg = Some(RawVal::from(arg).into());
                    let name = Some(name.into());

                    return Ok(Some(
                        T::default()
                            .with_idx(idx)
                            .with_tot(tot)
                            .with_name(name)
                            .with_arg(arg)
                            .with_style(style),
                    ));
                }
            }
        }
        Ok(None)
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<EmbeddedValuePlusStyle, MultiOpt<T, Set>>
    for CompleteGuess<'a, Set, Inv, Ser>
where
    Set: OptValidator,
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<MultiOpt<T, Set>>, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Argument;

        if self.arg.is_none() {
            if let Some(name) = &self.name {
                // strip the prefix before generate
                let validator = &self.set;
                let splited = validator.split(name.as_str()).map_err(Into::into)?;
                let char_indices = splited.1.char_indices().skip(2);
                let prefix_len = splited.0.len();
                let mut policy = MultiOpt::default().with_any_match(true);

                // make sure we using `chars.count`, not len()
                // check the name start 3th letter
                // for `--opt42` check the option like `--op t42`, `--opt 42`, `--opt4 2`
                for (char_idx, _) in char_indices {
                    let (name, arg) = name.split_at(prefix_len + char_idx);
                    let arg = Some(RawVal::from(arg).into());
                    let name = Some(name.into());

                    policy.add_sub_policy(
                        T::default()
                            .with_idx(idx)
                            .with_tot(tot)
                            .with_name(name)
                            .with_arg(arg)
                            .with_style(style),
                    );
                }
                return Ok(Some(policy));
            }
        }
        Ok(None)
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<CombinedOptionStyle, MultiOpt<T, Set>>
    for CompleteGuess<'a, Set, Inv, Ser>
where
    Set: OptValidator,
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<MultiOpt<T, Set>>, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Boolean;
        let arg = Some(ARef::new(RawVal::from(BOOL_TRUE)));

        if self.arg.is_none() {
            if let Some(name) = &self.name {
                // strip the prefix before generate
                let option = name.as_str();
                let validator = &self.set;
                let splited = validator.split(option).map_err(Into::into)?;

                if splited.1.chars().count() > 1 {
                    let mut policy = MultiOpt::default().with_any_match(false);

                    for ch in splited.1.chars() {
                        policy.add_sub_policy(
                            T::default()
                                .with_idx(idx)
                                .with_tot(tot)
                                .with_name(Some(format!("{}{}", splited.0, ch).into()))
                                .with_arg(arg.clone())
                                .with_style(style),
                        );
                    }
                    return Ok(Some(policy));
                }
            }
        }
        Ok(None)
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<BooleanStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        if self.arg.is_none() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.tot)
                        .with_name(Some(name.clone()))
                        .with_arg(Some(ARef::new(RawVal::from(BOOL_TRUE))))
                        .with_style(Style::Boolean),
                ));
            }
        }
        Ok(None)
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<FlagStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        if self.arg.is_none() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.tot)
                        .with_name(Some(name.clone()))
                        .with_arg(None)
                        .with_style(Style::Flag),
                ));
            }
        }
        Ok(None)
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<MainStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Main;
        let name = self.name.clone();
        let args = self.ctx.args().clone();
        let arg = args.get(idx).map(|v| v.clone().into());

        Ok(Some(
            T::default()
                .with_idx(idx)
                .with_arg(arg)
                .with_args(args)
                .with_name(name)
                .with_tot(tot)
                .with_style(style),
        ))
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<PosStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Pos;
        let name = self.name.clone();
        let args = self.ctx.args().clone();
        let arg = args.get(idx).map(|v| v.clone().into());

        Ok(Some(
            T::default()
                .with_idx(idx)
                .with_arg(arg)
                .with_args(args)
                .with_name(name)
                .with_tot(tot)
                .with_style(style),
        ))
    }
}

impl<'a, Set, Inv, Ser, T> GuessPolicy<CmdStyle, T> for CompleteGuess<'a, Set, Inv, Ser>
where
    T: Default + PolicyBuild,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.tot;
        let style = Style::Cmd;
        let name = self.name.clone();
        let args = self.ctx.args().clone();
        let arg = Some(RawVal::from(BOOL_TRUE).into());

        Ok(Some(
            T::default()
                .with_idx(idx)
                .with_arg(arg)
                .with_args(args)
                .with_name(name)
                .with_tot(tot)
                .with_style(style),
        ))
    }
}

impl<'a, 'b, Set, Inv, Ser> CompleteGuess<'a, Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'b, Set, Ser>,
{
    pub fn r#match<T>(&mut self, policy: &mut T, consume: bool) -> Result<bool, Error>
    where
        T: PolicyConfig + MatchPolicy<Set = Set>,
    {
        let uids = self.set.keys();

        for uid in uids {
            // if overload is true select all the option may match the `policy`
            if !policy.filter(uid, self.set) {
                if let Err(e) = policy.r#match(uid, self.set, false, consume) {
                    let e = e.into();

                    if !e.is_failure() {
                        return Err(e);
                    }
                }
            }
        }
        Ok(policy.matched())
    }

    fn match_multi(
        &mut self,
        policy: &mut MultiOpt<SingleOpt<Set>, Set>,
        consume: bool,
    ) -> Result<bool, Error> {
        let uids = self.set.keys();
        let any_match = policy.any_match();

        for sub_policy in policy.sub_policys_mut() {
            // process all uids with each policy first
            for uid in uids.iter() {
                if !sub_policy.filter(*uid, self.set) {
                    if let Err(e) = sub_policy.r#match(*uid, self.set, false, consume) {
                        if !e.is_failure() {
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

    pub fn apply<T>(&mut self, policy: &mut T, all: bool) -> Result<bool, Error>
    where
        T: PolicyConfig + MatchPolicy<Set = Set>,
    {
        let mut result = false;

        if !all {
            // apply first
            if let Some(uid) = policy.uids().first() {
                let uid = *uid;

                result = true;
                policy.apply(uid, self.set).map_err(Into::into)?;
            }
        } else if !policy.uids().is_empty() {
            let uids = policy.uids().to_vec();

            result = true;
            for uid in uids {
                policy.apply(uid, self.set).map_err(Into::into)?;
            }
        }

        Ok(result)
    }

    pub fn apply_multi<T>(
        &mut self,
        policy: &mut MultiOpt<T, Set>,
        all: bool,
    ) -> Result<bool, Error>
    where
        T: PolicyConfig + MatchPolicy<Set = Set>,
    {
        let mut matched = false;
        let any_match = policy.any_match();

        for sub_policy in policy.sub_policys_mut().iter_mut() {
            if self.apply(sub_policy, all)? {
                matched = true;
                if any_match {
                    // any match, return current
                    break;
                }
            } else if !any_match {
                matched = false;
                break;
            }
        }
        Ok(matched)
    }
}
