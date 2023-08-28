use std::fmt::Debug;
use std::marker::PhantomData;

use crate::args::Args;
use crate::ctx::InnerCtx;
use crate::opt::Opt;
use crate::opt::Style;
use crate::set::Set;
use crate::set::SetOpt;
use crate::ARef;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

use super::MatchPolicy;
use super::PolicyBuild;
use super::PolicyConfig;
use super::PolicyInnerCtx;

pub struct SingleOpt<S> {
    name: Option<Str>,

    style: Style,

    arg: Option<ARef<RawVal>>,

    args: ARef<Args>,

    uids: Vec<Uid>,

    index: usize,

    total: usize,

    marker: PhantomData<S>,
}

impl<S> Clone for SingleOpt<S> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            style: self.style,
            arg: self.arg.clone(),
            args: self.args.clone(),
            uids: self.uids.clone(),
            index: self.index,
            total: self.total,
            marker: self.marker,
        }
    }
}

impl<S> Debug for SingleOpt<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleOpt")
            .field("name", &self.name)
            .field("style", &self.style)
            .field("arg", &self.arg)
            .field("args", &self.args)
            .field("uids", &self.uids)
            .field("index", &self.index)
            .field("total", &self.total)
            .finish()
    }
}

impl<S> Default for SingleOpt<S> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            style: Default::default(),
            arg: Default::default(),
            args: Default::default(),
            uids: Default::default(),
            index: Default::default(),
            total: Default::default(),
            marker: Default::default(),
        }
    }
}

impl<S> PolicyBuild for SingleOpt<S> {
    fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    fn with_idx(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    fn with_tot(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    fn with_arg(mut self, arg: Option<ARef<RawVal>>) -> Self {
        self.arg = arg;
        self
    }

    fn with_args(mut self, args: ARef<Args>) -> Self {
        self.args = args;
        self
    }
}

impl<S> PolicyConfig for SingleOpt<S> {
    fn idx(&self) -> usize {
        self.index
    }

    fn tot(&self) -> usize {
        self.total
    }

    fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    fn style(&self) -> Style {
        self.style
    }

    fn arg(&self) -> Option<ARef<RawVal>> {
        self.arg.clone()
    }

    fn uids(&self) -> &[Uid] {
        &self.uids
    }

    fn collect_ctx(&self) -> Option<PolicyInnerCtx> {
        (!self.uids.is_empty()).then(|| PolicyInnerCtx {
            uids: self.uids().to_vec(),
            inner_ctx: InnerCtx::default()
                .with_idx(self.idx())
                .with_total(self.tot())
                .with_name(self.name().cloned())
                .with_arg(self.arg())
                .with_style(self.style()),
        })
    }
}

impl<S> SingleOpt<S> {
    pub fn clone_arg(&self) -> Option<ARef<RawVal>> {
        self.arg.clone()
    }

    pub fn set_uid(&mut self, uid: Uid) {
        self.uids.push(uid);
    }
}

impl<S> MatchPolicy for SingleOpt<S>
where
    S: Set,
    SetOpt<S>: Opt,
{
    type Set = S;

    type Ret = bool;

    type Error = Error;

    fn reset(&mut self) -> &mut Self {
        self.uids.clear();
        self
    }

    fn matched(&self) -> bool {
        !self.uids.is_empty()
    }

    fn undo(&mut self, uid: Uid, set: &mut Self::Set) -> Result<(), Self::Error> {
        if let Some(opt) = set.get_mut(uid) {
            opt.set_matched(false);
        }
        Ok(())
    }

    fn apply(&mut self, uid: Uid, set: &mut Self::Set) -> Result<(), Self::Error> {
        if let Some(opt) = set.get_mut(uid) {
            opt.set_matched(true);
        }
        Ok(())
    }

    fn filter(&mut self, uid: Uid, set: &mut Self::Set) -> bool {
        if let Some(opt) = set.get(uid) {
            !(opt.mat_style(Style::Argument)
                || opt.mat_style(Style::Boolean)
                || opt.mat_style(Style::Combined)
                || opt.mat_style(Style::Flag))
        } else {
            true
        }
    }

    fn r#match(
        &mut self,
        uid: Uid,
        set: &mut Self::Set,
        overload: bool,
        consume: bool,
    ) -> Result<Self::Ret, Error> {
        if overload || !self.matched() {
            if let Some(opt) = set.get(uid) {
                let mut matched = opt.mat_style(self.style);

                if matched {
                    if !opt.ignore_name() {
                        matched = opt.mat_name(self.name());
                    }
                    if !opt.ignore_alias() && opt.alias().is_some() {
                        matched = matched || opt.mat_alias(self.name.as_ref().unwrap())
                    }
                    if !opt.ignore_index() {
                        matched = matched && {
                            if opt.index().is_some() {
                                opt.mat_index(Some((self.index, self.total)))
                            } else {
                                false
                            }
                        };
                    }
                }
                if matched {
                    if consume && self.arg.is_none() {
                        return Err(Error::sp_missing_opt_value(opt.hint()).with_uid(uid));
                    }
                    self.set_uid(uid);
                }
                return Ok(matched);
            }
        }
        Ok(false)
    }
}
