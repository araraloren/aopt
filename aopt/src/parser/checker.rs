use std::fmt::Debug;
use std::marker::PhantomData;

use crate::opt::Index;
use crate::opt::Opt;
use crate::opt::Style;
use crate::set::SetChecker;
use crate::set::SetOpt;
use crate::trace_log;
use crate::Error;
use crate::HashMap;
use crate::StrJoin;
use crate::Uid;

/// Check the option base on [`Style`].
/// The checker will used for option check of [`Policy`](crate::parser::Policy).
pub struct DefaultSetChecker<S>(PhantomData<S>);

impl<S> Clone for DefaultSetChecker<S> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<S> Debug for DefaultSetChecker<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultSetChecker").finish()
    }
}

impl<S> Default for DefaultSetChecker<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S> DefaultSetChecker<S>
where
    S: crate::set::Set,
    SetOpt<S>: Opt,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn clear(&mut self) {}

    pub fn opt<'a>(set: &'a S, id: &Uid) -> &'a SetOpt<S> {
        set.get(*id).unwrap()
    }
}

impl<S> SetChecker<S> for DefaultSetChecker<S>
where
    S: crate::set::Set,
    SetOpt<S>: Opt,
{
    type Error = Error;

    /// Check if we have [`Cmd`](crate::opt::Style::Cmd),
    /// then no force required [`Pos`](crate::opt::Style::Pos)@1 allowed.
    fn pre_check(&self, set: &mut S) -> Result<bool, Error> {
        let has_cmd = set.iter().any(|opt| opt.mat_style(Style::Cmd));

        const MAX_INDEX: usize = usize::MAX;

        trace_log!("Pre Check {{has_cmd: {}}}", has_cmd);
        if has_cmd {
            for opt in set.iter() {
                if opt.mat_style(Style::Pos) {
                    if let Some(index) = opt.index() {
                        let index = index.calc_index(1, MAX_INDEX).unwrap_or(MAX_INDEX);

                        if index == 1 && opt.force() {
                            // if we have cmd, can not have force required POS @1
                            return Err(Error::unexcepted_pos_if_has_cmd().with_uid(opt.uid()));
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    /// Call the [`valid`](crate::opt::Opt::valid) check the
    /// options([`Argument`](crate::opt::Style::Argument),
    /// [`Boolean`](crate::opt::Style::Boolean), [`Combined`](crate::opt::Style::Combined)),
    /// [`Flag`](crate::opt::Style::Flag)
    fn opt_check(&self, set: &mut S) -> Result<bool, Error> {
        trace_log!("Opt Check, call valid on all Opt ...");
        for opt in set.iter().filter(|opt| {
            opt.mat_style(Style::Argument)
                || opt.mat_style(Style::Boolean)
                || opt.mat_style(Style::Combined)
                || opt.mat_style(Style::Flag)
        }) {
            if !opt.valid() {
                return Err(Error::raise_sp_opt_require(opt.hint()).with_uid(opt.uid()));
            }
        }
        Ok(true)
    }

    /// Check if the [`Pos`](crate::opt::Style::Pos) is valid, it must be set if it is force reuqired.
    fn pos_check(&self, set: &mut S) -> Result<bool, Error> {
        let mut index_map = HashMap::<usize, Vec<Uid>>::default();
        let mut float_vec: Vec<Uid> = vec![];

        const MAX_INDEX: usize = usize::MAX;

        for opt in set.iter() {
            if opt.mat_style(Style::Pos) {
                if let Some(index) = opt.index() {
                    match index {
                        Index::Forward(cnt) => {
                            if let Some(index) = index.calc_index(*cnt, MAX_INDEX) {
                                let entry = index_map.entry(index).or_default();
                                entry.push(opt.uid());
                            }
                        }
                        Index::List(v) => {
                            for index in v {
                                let entry = index_map.entry(*index).or_default();
                                entry.push(opt.uid());
                            }
                        }
                        Index::Range(start, Some(end)) => {
                            for index in *start..*end {
                                let entry = index_map.entry(index).or_default();
                                entry.push(opt.uid());
                            }
                        }
                        Index::Backward(_)
                        | Index::Except(_)
                        | Index::Range(_, _)
                        | Index::AnyWhere => {
                            float_vec.push(opt.uid());
                        }
                        Index::Null => {}
                    }
                }
            }
        }
        let mut names = vec![];

        trace_log!("Pos Check, index: {{{index_map:?}}}, float: {{{float_vec:?}}}");
        for (_, uids) in index_map {
            // if any of POS is force required, then it must set by user
            let mut pos_valid = true;

            for uid in uids.iter() {
                let opt = Self::opt(set, uid);
                let opt_valid = opt.valid();

                pos_valid = pos_valid && opt_valid;
                if !opt_valid {
                    names.push(opt.hint().to_owned());
                }
            }
            if !pos_valid {
                return Err(Error::raise_sp_pos_require(names.join(" | ")).with_uid(uids[0]));
            }
            names.clear();
        }
        if !float_vec.is_empty() {
            float_vec
                .iter()
                .filter(|&uid| !Self::opt(set, uid).valid())
                .for_each(|uid| {
                    names.push(Self::opt(set, uid).hint().clone());
                });
            if !names.is_empty() {
                return Err(Error::raise_sp_pos_require(names.join(" | ")).with_uid(float_vec[0]));
            }
        }
        Ok(true)
    }

    /// Return true if any one of [`Cmd`](Style::Cmd) matched.
    /// Return true if no [`Cmd`](Style::Cmd) exists.
    fn cmd_check(&self, set: &mut S) -> Result<bool, Error> {
        let mut names = vec![];
        let mut valid = false;
        let mut uids = vec![];

        for opt in set.iter() {
            if opt.mat_style(Style::Cmd) {
                valid = valid || opt.valid();
                if valid {
                    break;
                } else {
                    uids.push(opt.uid());
                    names.push(opt.hint().to_owned());
                }
            }
        }
        trace_log!("Cmd Check, any one of the cmd matched: {}", valid);
        if !valid && !names.is_empty() {
            return Err(Error::raise_sp_cmd_require(names.join(" | ")).with_uid(uids[0]));
        }
        Ok(true)
    }

    /// Call [`valid`](crate::opt::Opt::valid) on options those style are [`Main`](Style::Main).
    fn post_check(&self, set: &mut S) -> Result<bool, Error> {
        trace_log!("Post Check, call valid on Main ...");
        Ok(set
            .iter()
            .filter(|opt| opt.mat_style(Style::Main))
            .all(|opt| opt.valid()))
    }
}
