use std::fmt::Debug;
use std::marker::PhantomData;

use crate::opt::Index;
use crate::opt::Opt;
use crate::opt::Style;
use crate::set::SetOpt;
use crate::trace_log;
use crate::Error;
use crate::HashMap;
use crate::StrJoin;
use crate::Uid;

/// Service which do option check in [`Policy`](crate::parser::Policy).
#[derive(Clone)]
pub struct SetChecker<S>(PhantomData<S>);

#[cfg(feature = "sync")]
unsafe impl<S> Send for SetChecker<S> {}

#[cfg(feature = "sync")]
unsafe impl<S> Sync for SetChecker<S> {}

impl<S> Debug for SetChecker<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckService").finish()
    }
}

impl<S> Default for SetChecker<S> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

impl<S> SetChecker<S> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }

    pub fn clear(&mut self) {}
}

impl<S> SetChecker<S>
where
    S: crate::set::Set,
    SetOpt<S>: Opt,
{
    pub fn opt<'a>(set: &'a S, id: &Uid) -> &'a SetOpt<S> {
        set.get(*id).unwrap()
    }

    /// Check if we have [`Cmd`](crate::opt::Creator::cmd),
    /// then no force required [`Pos`](crate::opt::Creator::pos)@1 allowed.
    pub fn pre_check(&self, set: &mut S) -> Result<bool, Error> {
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
                            return Err(Error::con_can_not_insert_pos());
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    pub fn opt_check(&self, set: &mut S) -> Result<bool, Error> {
        trace_log!("Opt Check, call valid on all Opt ...");
        for opt in set.iter().filter(|opt| {
            opt.mat_style(Style::Argument)
                || opt.mat_style(Style::Boolean)
                || opt.mat_style(Style::Combined)
        }) {
            if !opt.valid() {
                return Err(Error::sp_opt_force_require(opt.hint()));
            }
        }
        Ok(true)
    }

    /// Check if the [`Pos`](crate::opt::Creator::pos) is valid.
    ///
    /// For which [`Pos`](crate::opt::Creator::pos) is have fixed position,
    /// [`Pos`](crate::opt::Creator::pos) has same position are replaceble even it is force reuqired.
    ///
    /// For which [`Pos`](crate::opt::Creator::pos) is have floating position, it must be set if it is force reuqired.
    pub fn pos_check(&self, set: &mut S) -> Result<bool, Error> {
        // for POS has certainty position, POS has same position are replaceble even it is force reuqired.
        let mut index_map = HashMap::<usize, Vec<Uid>>::default();
        // for POS has uncertainty position, it must be set if it is force reuqired
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
        for (_, uids) in index_map.iter() {
            // if any of POS is force required, then it must set by user
            let mut pos_valid = true;

            for uid in uids {
                let opt = Self::opt(set, uid);
                let opt_valid = opt.valid();

                pos_valid = pos_valid && opt_valid;
                if !opt_valid {
                    names.push(opt.hint().to_owned());
                }
            }
            if !pos_valid {
                return Err(Error::sp_pos_force_require(names.join(" | ")));
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
                return Err(Error::sp_pos_force_require(names.join(" | ")));
            }
        }
        Ok(true)
    }

    pub fn cmd_check(&self, set: &mut S) -> Result<bool, Error> {
        let mut names = vec![];
        let mut valid = false;

        for opt in set.iter() {
            if opt.mat_style(Style::Cmd) {
                valid = valid || opt.valid();
                if valid {
                    break;
                } else {
                    names.push(opt.hint().to_owned());
                }
            }
        }
        trace_log!("Cmd Check, any one of the cmd matched: {}", valid);
        if !valid && !names.is_empty() {
            return Err(Error::sp_cmd_force_require(names.join(" | ")));
        }
        Ok(true)
    }

    pub fn post_check(&self, set: &mut S) -> Result<bool, Error> {
        trace_log!("Post Check, call valid on Main ...");
        Ok(set
            .iter()
            .filter(|opt| opt.mat_style(Style::Main))
            .all(|opt| opt.valid()))
    }
}
