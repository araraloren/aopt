use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::args::Args;
use crate::ctx::InnerCtx;
use crate::opt::Opt;
use crate::opt::Style;
use crate::set::Set;
use crate::set::SetOpt;
use crate::Error;
use crate::Uid;

use super::MatchPolicy;
use super::PolicyBuild;
use super::PolicyConfig;
use super::PolicyInnerCtx;

pub struct SingleNonOpt<'a, S> {
    name: Option<Cow<'a, str>>,

    style: Style,

    arg: Option<Cow<'a, OsStr>>,

    args: Args<'a>,

    uids: Vec<Uid>,

    index: usize,

    total: usize,

    marker: PhantomData<S>,
}

impl<S> Clone for SingleNonOpt<'_, S> {
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

impl<S> Debug for SingleNonOpt<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleNOA")
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

impl<S> Default for SingleNonOpt<'_, S> {
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

impl<'a, S> PolicyBuild<'a> for SingleNonOpt<'a, S> {
    fn with_name(mut self, name: Option<Cow<'a, str>>) -> Self {
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

    fn with_arg(mut self, arg: Option<Cow<'a, OsStr>>) -> Self {
        self.arg = arg;
        self
    }

    fn with_args(mut self, args: Args<'a>) -> Self {
        self.args = args;
        self
    }
}

impl<'a, S> PolicyConfig<'a> for SingleNonOpt<'a, S> {
    fn idx(&self) -> usize {
        self.index
    }

    fn tot(&self) -> usize {
        self.total
    }

    fn name(&self) -> Option<&Cow<'a, str>> {
        self.name.as_ref()
    }

    fn style(&self) -> Style {
        self.style
    }

    fn arg(&self) -> Option<&Cow<'a, OsStr>> {
        self.arg.as_ref()
    }

    fn uids(&self) -> &[Uid] {
        &self.uids
    }

    fn collect_ctx(&self) -> Option<PolicyInnerCtx<'a>> {
        (!self.uids.is_empty()).then(|| PolicyInnerCtx {
            uids: self.uids().to_vec(),
            inner_ctx: InnerCtx::default()
                .with_idx(self.idx())
                .with_total(self.tot())
                .with_name(self.name().cloned())
                .with_arg(self.arg().cloned())
                .with_style(self.style()),
        })
    }
}

impl<'a, S> SingleNonOpt<'a, S> {
    pub fn clone_arg(&self) -> Option<Cow<'a, OsStr>> {
        self.arg.clone()
    }

    pub fn set_uid(&mut self, uid: Uid) {
        self.uids.push(uid);
    }

    pub fn reset_arg(mut self) -> Self {
        self.arg = self.args.get(self.idx()).cloned();
        self
    }
}

impl<'a, S> MatchPolicy for SingleNonOpt<'a, S>
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
            !(opt.mat_style(Style::Cmd) || opt.mat_style(Style::Main) || opt.mat_style(Style::Pos))
        } else {
            true
        }
    }

    fn r#match(
        &mut self,
        uid: Uid,
        set: &mut Self::Set,
        _overload: bool,
        _consume: bool,
    ) -> Result<Self::Ret, Error> {
        if let Some(opt) = set.get(uid) {
            let mut matched = opt.mat_style(self.style);

            if matched {
                if !opt.ignore_name() {
                    // FIXME
                    //matched = matched && opt.mat_name(self.name());
                }
                if !opt.ignore_alias() && opt.alias().is_some() {
                    if let Some(name) = &self.name {
                        // FIXME
                        //matched = matched || opt.mat_alias(name);
                    }
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
                self.set_uid(uid);
            }
            Ok(matched)
        } else {
            Ok(false)
        }
    }
}
