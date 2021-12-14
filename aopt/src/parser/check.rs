use std::collections::HashMap;

use super::Parser;

use crate::err::Error;
use crate::err::Result;
use crate::opt::Style;
use crate::set::Set;
use crate::uid::Uid;

/// Check the callback type in  parser .
pub fn default_pre_check<P: Parser>(set: &dyn Set, parser: &P) -> Result<bool> {
    for (uid, callback) in parser.callback_iter() {
        if let Some(opt) = set.get_opt(*uid) {
            if !opt.is_accept_callback_type(callback.borrow().to_callback_type()) {
                return Err(Error::opt_unsupport_callback_type(
                    opt.get_hint().as_ref(),
                    &format!("{:?}", callback.borrow().to_callback_type()),
                ));
            }
        } else {
            warn!(%uid, "callback has unknow option uid");
        }
    }
    Ok(true)
}

pub fn default_opt_check<P: Parser>(set: &dyn Set, _parser: &P) -> Result<bool> {
    for opt in set.opt_iter() {
        if opt.as_ref().match_style(Style::Boolean)
            || opt.as_ref().match_style(Style::Argument)
            || opt.as_ref().match_style(Style::Multiple)
        {
            opt.check()?;
        }
    }
    Ok(true)
}

pub fn default_nonopt_check<P: Parser>(set: &dyn Set, _parser: &P) -> Result<bool> {
    const MAX_INDEX: u64 = u64::MAX;

    let mut index_map: HashMap<u64, Vec<Uid>> = HashMap::new();

    for opt in set.opt_iter() {
        if opt.as_ref().match_style(Style::Pos)
            || opt.as_ref().match_style(Style::Cmd)
            || opt.as_ref().match_style(Style::Main)
        {
            if let Some(index) = opt.as_ref().get_index() {
                let index = index.calc_index(MAX_INDEX, 1).unwrap_or(MAX_INDEX);
                let entry = index_map.entry(index).or_insert(vec![]);

                entry.push(opt.as_ref().get_uid());
            }
        }
    }

    trace!(?index_map, "non-opt check information");

    let mut names = vec![];

    for (index, uids) in index_map.iter() {
        let valid;

        // <cmd1> <cmd2> <pos3> [pos4] [pos5]
        // any of thing at position 1
        if index == &1 || index == &0 {
            let mut cmd_count = 0;
            let mut cmd_valid = false;
            let mut pos_valid = true;
            let mut force_valid = false;

            for uid in uids {
                let opt = set.get_opt(*uid).unwrap();

                if opt.match_style(Style::Cmd) {
                    cmd_count += 1;
                    // set the cmd will valid the check
                    // if any of cmd is valid, break out
                    cmd_valid = cmd_valid || opt.check().is_ok();
                    if cmd_valid {
                        break;
                    }
                    names.push(opt.get_hint().to_owned());
                } else if opt.match_style(Style::Pos) {
                    let opt_valid = opt.check().is_ok();

                    pos_valid = pos_valid && opt_valid;
                    if opt_valid && !opt.get_optional() {
                        force_valid = true;
                        names.push(opt.get_hint().to_owned());
                    }
                }
            }

            debug!(%cmd_valid, %pos_valid, %force_valid, "in default nonopt-check");

            // if we have CMD, then the CMD must be set or any POS is set
            // if all nonopt @1 are POS, it's normally like @2..
            if cmd_count > 0 {
                valid = cmd_valid || (pos_valid && force_valid);
            } else {
                valid = pos_valid;
            }
        } else {
            // <pos1> [pos2] [pos3] [pos4] [pos5]
            // if any of POS is force required, then it must set by user
            let mut pos_valid = true;

            for uid in uids {
                let opt = set.get_opt(*uid).unwrap();
                let opt_valid = opt.check().is_ok();

                pos_valid = pos_valid && opt_valid;
                if !opt_valid {
                    names.push(opt.get_hint().to_owned());
                }
            }
            debug!(%pos_valid, "in default nonopt-check");
            valid = pos_valid;
        }
        if !valid {
            return Err(Error::sp_pos_force_require(*index, names.join(" | ")));
        }
        names.clear();
    }

    Ok(true)
}

pub fn default_post_check<P: Parser>(_set: &dyn Set, _parser: &P) -> Result<bool> {
    Ok(true)
}
