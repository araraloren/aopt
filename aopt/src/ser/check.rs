use std::marker::PhantomData;

use super::Service;
use crate::astr;
use crate::opt::Opt;
use crate::opt::OptIndex;
use crate::opt::OptStyle;
use crate::set::Set;
use crate::Error;
use crate::HashMap;
use crate::StrJoin;
use crate::Uid;

#[derive(Debug, Default)]
pub struct CheckService<S, V>(PhantomData<(S, V)>)
where
    S: Set;

impl<S, V> CheckService<S, V>
where
    S: Set,
    S::Opt: Opt,
{
    pub fn new() -> Self {
        Self(PhantomData::default())
    }

    pub fn opt<'a>(set: &'a S, id: &Uid) -> &'a dyn Opt {
        set.get(*id).unwrap()
    }

    pub fn pre_check(&self, set: &mut S) -> Result<bool, Error> {
        let has_cmd = set
            .keys()
            .iter()
            .any(|key| Self::opt(set, key).match_style(OptStyle::Cmd));

        const MAX_INDEX: usize = usize::MAX;

        if has_cmd {
            for key in set.keys() {
                let opt = Self::opt(set, key);

                if opt.match_style(OptStyle::Pos) {
                    if let Some(index) = opt.get_index() {
                        let index = index.calc_index(MAX_INDEX, 1).unwrap_or(MAX_INDEX);
                        if index == 1 && !opt.get_optional() {
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
        Ok(set
            .keys()
            .iter()
            .filter(|v| {
                let opt = Self::opt(set, *v);
                opt.match_style(OptStyle::Argument)
                    || opt.match_style(OptStyle::Boolean)
                    || opt.match_style(OptStyle::Combined)
            })
            .all(|v| Self::opt(set, v).check()))
    }

    /// Check if the POS is valid.
    /// For which POS is have certainty position, POS has same position are replaceble even it is force reuqired.
    /// For which POS is have uncertainty position, it must be set if it is force reuqired.
    pub fn pos_check(&self, set: &mut S) -> Result<bool, Error> {
        // for POS has certainty position, POS has same position are replaceble even it is force reuqired.
        let mut index_map = HashMap::<usize, Vec<Uid>>::default();
        // for POS has uncertainty position, it must be set if it is force reuqired
        let mut float_vec: Vec<Uid> = vec![];

        for key in set.keys() {
            let opt = Self::opt(set, key);

            if opt.match_style(OptStyle::Pos) {
                if let Some(index) = opt.get_index() {
                    match index {
                        OptIndex::Forward(_) | OptIndex::Backward(_) => {
                            if let Some(index) = index.calc_index(usize::MAX, 1) {
                                let entry = index_map.entry(index).or_insert(vec![]);
                                entry.push(opt.get_uid());
                            }
                        }
                        OptIndex::List(v) => {
                            for index in v {
                                let entry = index_map.entry(*index).or_insert(vec![]);
                                entry.push(opt.get_uid());
                            }
                        }
                        OptIndex::Except(_)
                        | OptIndex::Greater(_)
                        | OptIndex::Less(_)
                        | OptIndex::AnyWhere => {
                            float_vec.push(opt.get_uid());
                        }
                        OptIndex::Null => {}
                    }
                }
            }
        }
        let mut names = vec![];

        for (_, uids) in index_map.iter() {
            // if any of POS is force required, then it must set by user
            let mut pos_valid = true;

            for uid in uids {
                let opt = Self::opt(set, uid);
                let opt_valid = opt.check();

                pos_valid = pos_valid && opt_valid;
                if !opt_valid {
                    names.push(opt.get_hint().to_owned());
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
                .filter(|&uid| !Self::opt(set, uid).check())
                .for_each(|uid| {
                    names.push(Self::opt(set, uid).get_hint());
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

        for key in set.keys() {
            let opt = Self::opt(set, key);

            if opt.match_style(OptStyle::Cmd) {
                valid = valid || opt.check();
                if valid {
                    break;
                } else {
                    names.push(opt.get_hint().to_owned());
                }
            }
        }
        if !valid && !names.is_empty() {
            return Err(Error::sp_cmd_force_require(names.join(" | ")));
        }
        Ok(true)
    }

    pub fn post_check(&self, set: &mut S) -> Result<bool, Error> {
        Ok(set
            .keys()
            .iter()
            .filter(|v| Self::opt(set, *v).match_style(OptStyle::Main))
            .all(|v| Self::opt(set, v).check()))
    }
}

impl<S, V> Service for CheckService<S, V>
where
    S: Set,
{
    fn service_name() -> crate::Str {
        astr("CheckService")
    }
}
