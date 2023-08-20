use std::fmt::Debug;
use std::marker::PhantomData;

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

pub struct SingleOpt<S> {
    name: Str,

    style: Style,

    arg: Option<ARef<RawVal>>,

    uids: Vec<Uid>,

    consume: bool,

    index: usize,

    total: usize,

    marker: PhantomData<S>,
}

impl<S> Clone for SingleOpt<S> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            style: self.style.clone(),
            arg: self.arg.clone(),
            uids: self.uids.clone(),
            consume: self.consume.clone(),
            index: self.index.clone(),
            total: self.total.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<S> Debug for SingleOpt<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleOpt")
            .field("name", &self.name)
            .field("style", &self.style)
            .field("arg", &self.arg)
            .field("uids", &self.uids)
            .field("consume", &self.consume)
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
            uids: Default::default(),
            consume: Default::default(),
            index: Default::default(),
            total: Default::default(),
            marker: Default::default(),
        }
    }
}

impl<S> SingleOpt<S> {
    pub fn with_name(mut self, name: Str) -> Self {
        self.name = name;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_idx(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_total(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn with_consume(mut self, consume: bool) -> Self {
        self.consume = consume;
        self
    }

    pub fn with_arg(mut self, argument: Option<ARef<RawVal>>) -> Self {
        self.arg = argument;
        self
    }

    pub fn name(&self) -> Option<&Str> {
        Some(&self.name)
    }

    pub fn idx(&self) -> usize {
        self.index
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn clone_arg(&self) -> Option<ARef<RawVal>> {
        self.arg.clone()
    }

    pub fn uids(&self) -> &[Uid] {
        &self.uids
    }

    pub fn set_uid(&mut self, uid: Uid) {
        self.uids.push(uid);
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn arg(&self) -> Option<&RawVal> {
        self.arg.as_ref().map(|v| v.as_ref())
    }

    pub fn is_consume(&self) -> bool {
        self.consume
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

    fn r#match(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Self::Ret, Error> {
        if let Some(opt) = set.get(uid) {
            let mut matched = opt.mat_style(self.style);

            if matched {
                if !opt.ignore_name() {
                    matched = opt.mat_name(self.name());
                }
                if !opt.ignore_alias() && opt.alias().is_some() {
                    matched = matched || opt.mat_alias(&self.name)
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
                if self.is_consume() && self.arg.is_none() {
                    return Err(Error::sp_missing_opt_value(opt.hint()).with_uid(uid));
                }
                self.set_uid(uid);
            }
            Ok(matched)
        } else {
            Ok(false)
        }
    }
}
