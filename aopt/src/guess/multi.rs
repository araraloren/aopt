use std::fmt::Debug;

use super::{MatchPolicy, SingleOpt};

use crate::opt::Opt;
use crate::opt::Style;
use crate::set::Set;
use crate::set::SetOpt;
use crate::Error;
use crate::Uid;

pub struct MultiOpt<S> {
    any_match: bool,

    single_opt: Vec<SingleOpt<S>>,
}

impl<S> Debug for MultiOpt<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiOpt")
            .field("any_match", &self.any_match)
            .field("single_opt", &self.single_opt)
            .finish()
    }
}

impl<S> Clone for MultiOpt<S> {
    fn clone(&self) -> Self {
        Self {
            any_match: self.any_match.clone(),
            single_opt: self.single_opt.clone(),
        }
    }
}

impl<S> Default for MultiOpt<S> {
    fn default() -> Self {
        Self {
            any_match: Default::default(),
            single_opt: Default::default(),
        }
    }
}

impl<S> MultiOpt<S> {
    pub fn with_any_match(mut self, any_match: bool) -> Self {
        self.any_match = any_match;
        self
    }

    pub fn with_sub_policy(mut self, single_opt: Vec<SingleOpt<S>>) -> Self {
        self.single_opt = single_opt;
        self
    }

    pub fn any_match(&self) -> bool {
        self.any_match
    }

    pub fn add_sub_policy(&mut self, policy: SingleOpt<S>) -> &mut Self {
        self.single_opt.push(policy);
        self
    }

    pub fn sub_policys(&self) -> &[SingleOpt<S>] {
        &self.single_opt
    }

    pub fn sub_policys_mut(&mut self) -> &mut [SingleOpt<S>] {
        &mut self.single_opt
    }

    pub fn len(&self) -> usize {
        self.single_opt.len()
    }

    pub fn is_empty(&self) -> bool {
        self.single_opt.is_empty()
    }
}

impl<S> MatchPolicy for MultiOpt<S>
where
    S: Set,
    SetOpt<S>: Opt,
{
    type Set = S;

    type Ret = Option<usize>;

    type Error = Error;

    fn reset(&mut self) -> &mut Self {
        self.single_opt.iter_mut().for_each(|v| {
            v.reset();
        });
        self
    }

    fn matched(&self) -> bool {
        if self.any_match {
            self.single_opt.iter().any(SingleOpt::matched)
        } else {
            self.single_opt.iter().all(SingleOpt::matched)
        }
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

    fn r#match(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Self::Ret, Self::Error> {
        for (index, single_opt) in self.single_opt.iter_mut().enumerate() {
            if single_opt.r#match(uid, set)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
}
