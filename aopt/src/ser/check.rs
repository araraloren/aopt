use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::trace;

use super::Service;
use crate::astr;
use crate::opt::Index;
use crate::opt::Opt;
use crate::opt::Style;
use crate::Error;
use crate::HashMap;
use crate::StrJoin;
use crate::Uid;

pub struct CheckService<Set>(PhantomData<Set>);

impl<Set> Debug for CheckService<Set> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckService").finish()
    }
}

impl<Set> Default for CheckService<Set> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

impl<Set> CheckService<Set> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

impl<Set> CheckService<Set>
where
    Set: crate::set::Set,
    Set::Opt: Opt,
{
    pub fn opt<'a>(set: &'a Set, id: &Uid) -> &'a dyn Opt {
        set.get(*id).unwrap()
    }

    /// Check if we have [`Cmd`](crate::opt::CmdCreator),
    /// then no force required [`Pos`](crate::opt::PosCreator)@1 allowed.
    pub fn pre_check(&self, set: &mut Set) -> Result<bool, Error> {
        let has_cmd = set
            .keys()
            .iter()
            .any(|key| Self::opt(set, key).mat_style(Style::Cmd));

        const MAX_INDEX: usize = usize::MAX;

        trace!("Pre Check {{has_cmd: {}}}", has_cmd);
        if has_cmd {
            for key in set.keys() {
                let opt = Self::opt(set, key);

                if opt.mat_style(Style::Pos) {
                    if let Some(index) = opt.idx() {
                        let index = index.calc_index(MAX_INDEX, 1).unwrap_or(MAX_INDEX);
                        if index == 1 && !opt.optional() {
                            // if we have cmd, can not have force required POS @1
                            return Err(Error::con_can_not_insert_pos());
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    pub fn opt_check(&self, set: &mut Set) -> Result<bool, Error> {
        trace!("Opt Check, call valid on all Opt ...");
        for id in set.keys().iter().filter(|v| {
            let opt = Self::opt(set, *v);
            opt.mat_style(Style::Argument)
                || opt.mat_style(Style::Boolean)
                || opt.mat_style(Style::Combined)
        }) {
            let opt = Self::opt(set, id);
            if !opt.valid() {
                return Err(Error::sp_opt_force_require(opt.hint()));
            }
        }
        Ok(true)
    }

    /// Check if the POS is valid.
    /// For which POS is have certainty position, POS has same position are replaceble even it is force reuqired.
    /// For which POS is have uncertainty position, it must be set if it is force reuqired.
    pub fn pos_check(&self, set: &mut Set) -> Result<bool, Error> {
        // for POS has certainty position, POS has same position are replaceble even it is force reuqired.
        let mut index_map = HashMap::<usize, Vec<Uid>>::default();
        // for POS has uncertainty position, it must be set if it is force reuqired
        let mut float_vec: Vec<Uid> = vec![];

        for key in set.keys() {
            let opt = Self::opt(set, key);

            if opt.mat_style(Style::Pos) {
                if let Some(index) = opt.idx() {
                    match index {
                        Index::Forward(_) | Index::Backward(_) => {
                            if let Some(index) = index.calc_index(usize::MAX, 1) {
                                let entry = index_map.entry(index).or_insert(vec![]);
                                entry.push(opt.uid());
                            }
                        }
                        Index::List(v) => {
                            for index in v {
                                let entry = index_map.entry(*index).or_insert(vec![]);
                                entry.push(opt.uid());
                            }
                        }
                        Index::Except(_) | Index::Range(_, _) | Index::AnyWhere => {
                            float_vec.push(opt.uid());
                        }
                        Index::Null => {}
                    }
                }
            }
        }
        let mut names = vec![];

        trace!(
            "Pos Check, index: {{{:?}}}, float: {{{:?}}}",
            index_map,
            float_vec
        );
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

    pub fn cmd_check(&self, set: &mut Set) -> Result<bool, Error> {
        let mut names = vec![];
        let mut valid = false;

        for key in set.keys() {
            let opt = Self::opt(set, key);

            if opt.mat_style(Style::Cmd) {
                valid = valid || opt.valid();
                if valid {
                    break;
                } else {
                    names.push(opt.hint().to_owned());
                }
            }
        }
        trace!("Cmd Check, any one of the cmd matched: {}", valid);
        if !valid && !names.is_empty() {
            return Err(Error::sp_cmd_force_require(names.join(" | ")));
        }
        Ok(true)
    }

    pub fn post_check(&self, set: &mut Set) -> Result<bool, Error> {
        trace!("Post Check, call valid on Main ...");
        Ok(set
            .keys()
            .iter()
            .filter(|v| Self::opt(set, *v).mat_style(Style::Main))
            .all(|v| Self::opt(set, v).valid()))
    }
}

impl<Set> Service for CheckService<Set> {
    fn service_name() -> crate::Str {
        astr("CheckService")
    }
}
