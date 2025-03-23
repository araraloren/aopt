use std::borrow::Cow;
use std::ffi::OsStr;

use crate::ctx::Ctx;
use crate::ctx::HandlerCollection;
use crate::opt::Style;
use crate::opt::BOOL_TRUE;
use crate::parser::UserStyle;
use crate::set::OptValidator;
use crate::str::CowStrUtils;
use crate::Error;
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
pub struct CompleteGuess<'a, 'b, S, Inv> {
    pub idx: usize,

    pub total: usize,

    pub arg: Option<Cow<'b, OsStr>>,

    pub name: Option<Cow<'b, str>>,

    pub next: Option<Cow<'b, OsStr>>,

    pub ctx: &'a mut Ctx<'b>,

    pub set: &'a mut S,

    pub inv: &'a mut Inv,
}

impl<'a, 'b, S, Inv> CompleteGuess<'a, 'b, S, Inv> {
    pub fn new(ctx: &'a mut Ctx<'b>, set: &'a mut S, inv: &'a mut Inv) -> Self {
        Self {
            idx: 0,
            total: 0,
            arg: None,
            name: None,
            next: None,
            ctx,
            set,
            inv,
        }
    }

    pub fn set_ctx(&mut self, ctx: &'a mut Ctx<'b>) -> &mut Self {
        self.ctx = ctx;
        self
    }

    pub fn set_optset(&mut self, set: &'a mut S) -> &mut Self {
        self.set = set;
        self
    }

    pub fn set_inv(&mut self, inv: &'a mut Inv) -> &mut Self {
        self.inv = inv;
        self
    }

    pub fn set_idx(&mut self, idx: usize) -> &mut Self {
        self.idx = idx;
        self
    }

    pub fn set_tot(&mut self, tot: usize) -> &mut Self {
        self.total = tot;
        self
    }

    pub fn set_arg(&mut self, arg: Option<Cow<'b, OsStr>>) -> &mut Self {
        self.arg = arg;
        self
    }

    pub fn set_name(&mut self, name: Option<Cow<'b, str>>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_next(&mut self, next: Option<Cow<'b, OsStr>>) -> &mut Self {
        self.next = next;
        self
    }

    pub fn with_ctx(mut self, ctx: &'a mut Ctx<'b>) -> Self {
        self.ctx = ctx;
        self
    }

    pub fn with_set(mut self, set: &'a mut S) -> Self {
        self.set = set;
        self
    }

    pub fn with_inv(mut self, inv: &'a mut Inv) -> Self {
        self.inv = inv;
        self
    }

    pub fn with_idx(mut self, idx: usize) -> Self {
        self.idx = idx;
        self
    }

    pub fn with_tot(mut self, tot: usize) -> Self {
        self.total = tot;
        self
    }

    pub fn with_arg(mut self, arg: Option<Cow<'b, OsStr>>) -> Self {
        self.arg = arg;
        self
    }

    pub fn with_name(mut self, name: Option<Cow<'b, str>>) -> Self {
        self.name = name;
        self
    }

    pub fn with_next(mut self, next: Option<Cow<'b, OsStr>>) -> Self {
        self.next = next;
        self
    }
}

impl<'c, S, Inv> CompleteGuess<'_, '_, S, Inv>
where
    S: crate::set::Set + OptValidator,
    Inv: HandlerCollection<'c, S>,
{
    pub fn guess_complete(&mut self, style: &UserStyle) -> Result<Option<SimpleMatRet>, Error> {
        let mut matched = false;
        let mut consume = false;

        match style {
            UserStyle::Main => {
                if let Some(mut policy) =
                    GuessPolicy::<MainStyle, SingleNonOpt<S>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, true)?;
                    }
                }
            }
            UserStyle::Pos => {
                if let Some(mut policy) =
                    GuessPolicy::<PosStyle, SingleNonOpt<S>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, true)?;
                    }
                }
            }
            UserStyle::Cmd => {
                if let Some(mut policy) =
                    GuessPolicy::<CmdStyle, SingleNonOpt<S>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, true)?;
                    }
                }
            }
            UserStyle::EqualWithValue => {
                if let Some(mut policy) =
                    GuessPolicy::<EqualWithValuStyle, SingleOpt<S>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::Argument => {
                if let Some(mut policy) =
                    GuessPolicy::<ArgumentStyle, SingleOpt<S>>::guess_policy(self)?
                {
                    consume = true;
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::EmbeddedValue => {
                if let Some(mut policy) =
                    GuessPolicy::<EmbeddedValueStyle, SingleOpt<S>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::EmbeddedValuePlus => {
                if let Some(mut policy) = GuessPolicy::<
                    EmbeddedValuePlusStyle,
                    MultiOpt<SingleOpt<S>, S>,
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
                    MultiOpt<SingleOpt<S>, S>,
                >::guess_policy(self)?
                {
                    if self.match_multi(&mut policy, consume)? {
                        matched = self.apply_multi(&mut policy, false)?;
                    }
                }
            }
            UserStyle::Boolean => {
                if let Some(mut policy) =
                    GuessPolicy::<BooleanStyle, SingleOpt<S>>::guess_policy(self)?
                {
                    if self.r#match(&mut policy, consume)? {
                        matched = self.apply(&mut policy, false)?;
                    }
                }
            }
            UserStyle::Flag => {
                if let Some(mut policy) =
                    GuessPolicy::<FlagStyle, SingleOpt<S>>::guess_policy(self)?
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

impl<'b, S, Inv, T> GuessPolicy<EqualWithValuStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        if self.arg.is_some() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.total)
                        .with_name(Some(name.clone()))
                        .with_arg(self.arg.clone())
                        .with_style(Style::Argument),
                ));
            }
        }
        Ok(None)
    }
}

impl<'b, S, Inv, T> GuessPolicy<ArgumentStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        if self.arg.is_none() && self.next.is_some() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.total)
                        .with_name(Some(name.clone()))
                        .with_arg(self.next.clone())
                        .with_style(Style::Argument),
                ));
            }
        }
        Ok(None)
    }
}

impl<'b, S, Inv, T> GuessPolicy<EmbeddedValueStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    S: OptValidator,
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.total;
        let style = Style::Argument;

        if self.arg.is_none() {
            if let Some(name) = &self.name {
                // strip the prefix before generate
                let validator = &self.set;
                let splited = validator.split(name).map_err(Into::into)?;
                let prefix_len = splited.0.len();

                // make sure we using `chars.count`, not len()
                // make sure the name length >= 2
                // only check first letter `--v42` ==> `--v 42`
                if let Some((char_idx, _)) = splited.1.char_indices().nth(1) {
                    let (name, arg) = name.split_at(prefix_len + char_idx);
                    let arg = Some(arg.to_os_str());
                    let name = Some(name);

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

impl<'b, S, Inv, T> GuessPolicy<EmbeddedValuePlusStyle, MultiOpt<T, S>>
    for CompleteGuess<'_, 'b, S, Inv>
where
    S: OptValidator,
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<MultiOpt<T, S>>, Self::Error> {
        let idx = self.idx;
        let tot = self.total;
        let style = Style::Argument;

        if self.arg.is_none() {
            if let Some(name) = &self.name {
                // strip the prefix before generate
                let validator = &self.set;
                let splited = validator.split(name).map_err(Into::into)?;
                let char_indices = splited.1.char_indices().skip(2);
                let prefix_len = splited.0.len();
                let mut policy = MultiOpt::default().with_any_match(true);

                // make sure we using `chars.count`, not len()
                // check the name start 3th letter
                // for `--opt42` check the option like `--op t42`, `--opt 42`, `--opt4 2`
                for (char_idx, _) in char_indices {
                    let (name, arg) = name.split_at(prefix_len + char_idx);
                    let arg = Some(arg.to_os_str());
                    let name = Some(name);

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

impl<'b, S, Inv, T> GuessPolicy<CombinedOptionStyle, MultiOpt<T, S>>
    for CompleteGuess<'_, 'b, S, Inv>
where
    S: OptValidator,
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<MultiOpt<T, S>>, Self::Error> {
        let idx = self.idx;
        let tot = self.total;
        let style = Style::Boolean;
        let arg = Some(Cow::Borrowed(OsStr::new(BOOL_TRUE)));

        if self.arg.is_none() {
            if let Some(name) = &self.name {
                // strip the prefix before generate
                let validator = &self.set;
                let splited = validator.split(name).map_err(Into::into)?;

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

impl<'b, S, Inv, T> GuessPolicy<BooleanStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let arg = Some(Cow::Borrowed(OsStr::new(BOOL_TRUE)));

        if self.arg.is_none() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.total)
                        .with_name(Some(name.clone()))
                        .with_arg(arg)
                        .with_style(Style::Boolean),
                ));
            }
        }
        Ok(None)
    }
}

impl<'b, S, Inv, T> GuessPolicy<FlagStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        if self.arg.is_none() {
            if let Some(name) = &self.name {
                return Ok(Some(
                    T::default()
                        .with_idx(self.idx)
                        .with_tot(self.total)
                        .with_name(Some(name.clone()))
                        .with_arg(None)
                        .with_style(Style::Flag),
                ));
            }
        }
        Ok(None)
    }
}

impl<'b, S, Inv, T> GuessPolicy<MainStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.total;
        let style = Style::Main;
        let name = self.name.clone();
        let args = self.ctx.args();
        let arg = args.get(idx).map(|v| Cow::Borrowed(*v));

        Ok(Some(
            T::default()
                .with_idx(idx)
                .with_arg(arg)
                .with_name(name)
                .with_tot(tot)
                .with_style(style),
        ))
    }
}

impl<'b, S, Inv, T> GuessPolicy<PosStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.total;
        let style = Style::Pos;
        let name = self.name.clone();
        let args = self.ctx.args();
        let arg = args.get(idx).map(|v| Cow::Borrowed(*v));

        Ok(Some(
            T::default()
                .with_idx(idx)
                .with_arg(arg)
                .with_name(name)
                .with_tot(tot)
                .with_style(style),
        ))
    }
}

impl<'b, S, Inv, T> GuessPolicy<CmdStyle, T> for CompleteGuess<'_, 'b, S, Inv>
where
    T: Default + PolicyBuild<'b>,
{
    type Error = Error;

    fn guess_policy(&mut self) -> Result<Option<T>, Self::Error> {
        let idx = self.idx;
        let tot = self.total;
        let style = Style::Cmd;
        let name = self.name.clone();
        let arg = Some(Cow::Borrowed(OsStr::new(BOOL_TRUE)));

        Ok(Some(
            T::default()
                .with_idx(idx)
                .with_arg(arg)
                .with_name(name)
                .with_tot(tot)
                .with_style(style),
        ))
    }
}

impl<'b, 'c, S, Inv> CompleteGuess<'_, 'b, S, Inv>
where
    S: crate::set::Set,
    Inv: HandlerCollection<'c, S>,
{
    pub fn r#match<T>(&mut self, policy: &mut T, consume: bool) -> Result<bool, Error>
    where
        T: PolicyConfig<'b> + MatchPolicy<Set = S>,
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
        policy: &mut MultiOpt<SingleOpt<S>, S>,
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
        T: PolicyConfig<'b> + MatchPolicy<Set = S>,
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

    pub fn apply_multi<T>(&mut self, policy: &mut MultiOpt<T, S>, all: bool) -> Result<bool, Error>
    where
        T: PolicyConfig<'b> + MatchPolicy<Set = S>,
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
