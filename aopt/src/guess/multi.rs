use std::fmt::Debug;
use std::marker::PhantomData;

use super::MatchPolicy;

use crate::opt::Opt;
use crate::opt::Style;
use crate::set::Set;
use crate::set::SetOpt;
use crate::Error;
use crate::Uid;

pub struct MultiOpt<T, S> {
    any_match: bool,

    sub_policys: Vec<T>,

    marker: PhantomData<S>,
}

impl<T, S> Debug for MultiOpt<T, S>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiOpt")
            .field("any_match", &self.any_match)
            .field("sub_policys", &self.sub_policys)
            .finish()
    }
}

impl<T, S> Clone for MultiOpt<T, S>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            any_match: self.any_match,
            sub_policys: self.sub_policys.clone(),
            marker: self.marker,
        }
    }
}

impl<T, S> Default for MultiOpt<T, S>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            any_match: Default::default(),
            sub_policys: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<T, S> MultiOpt<T, S> {
    pub fn with_any_match(mut self, any_match: bool) -> Self {
        self.any_match = any_match;
        self
    }

    pub fn with_sub_policy(mut self, single_opt: Vec<T>) -> Self {
        self.sub_policys = single_opt;
        self
    }

    pub fn any_match(&self) -> bool {
        self.any_match
    }

    pub fn add_sub_policy(&mut self, policy: T) -> &mut Self {
        self.sub_policys.push(policy);
        self
    }

    pub fn sub_policy(&self, idx: usize) -> Option<&T> {
        self.sub_policys.get(idx)
    }

    pub fn sub_policy_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.sub_policys.get_mut(idx)
    }

    pub fn sub_policys(&self) -> &[T] {
        &self.sub_policys
    }

    pub fn sub_policys_mut(&mut self) -> &mut [T] {
        &mut self.sub_policys
    }

    pub fn len(&self) -> usize {
        self.sub_policys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sub_policys.is_empty()
    }
}

impl<T, S> MatchPolicy for MultiOpt<T, S>
where
    S: Set,
    SetOpt<S>: Opt,
    T: MatchPolicy<Set = S, Ret = bool, Error = Error>,
{
    type Set = S;

    type Ret = Option<usize>;

    type Error = Error;

    fn reset(&mut self) -> &mut Self {
        self.sub_policys.iter_mut().for_each(|v| {
            v.reset();
        });
        self
    }

    fn matched(&self) -> bool {
        if self.any_match {
            self.sub_policys.iter().any(MatchPolicy::matched)
        } else {
            self.sub_policys.iter().all(MatchPolicy::matched)
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

    fn r#match(
        &mut self,
        uid: Uid,
        set: &mut Self::Set,
        overload: bool,
        consume: bool,
    ) -> Result<Self::Ret, Self::Error> {
        for (index, sub_policy) in self.sub_policys.iter_mut().enumerate() {
            if sub_policy.r#match(uid, set, overload, consume)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
}
