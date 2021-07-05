use std::collections::HashMap;

use super::Parser;
use crate::err::{Error, Result};
use crate::opt::Style;
use crate::set::Set;
use crate::uid::Uid;

pub fn default_pre_check<S: Set, P: Parser<S>>(set: &S, parser: &P) -> Result<bool> {
    for (uid, callback) in parser.callback_iter() {
        if let Some(opt) = set.get_opt(*uid) {
            if !opt.is_accept_callback_type(callback.borrow().to_callback_type()) {
                return Err(Error::InvalidOptionCallbackData(format!(
                    "invalid callback type {:?}",
                    callback
                )));
            }
        } else {
            return Err(Error::InvalidOptionCallbackData(format!(
                "invalid id {}",
                uid
            )));
        }
    }
    Ok(true)
}

pub fn default_opt_check<S: Set, P: Parser<S>>(set: &S, _parser: &P) -> Result<bool> {
    for opt in set.iter() {
        if opt.as_ref().match_style(Style::Boolean)
            || opt.as_ref().match_style(Style::Argument)
            || opt.as_ref().match_style(Style::Multiple)
        {
            opt.check()?;
        }
    }
    Ok(true)
}

pub fn default_nonopt_check<S: Set, P: Parser<S>>(set: &S, _parser: &P) -> Result<bool> {
    const MAX_INDEX: u64 = u64::MAX;

    let mut index_map: HashMap<u64, Vec<Uid>> = HashMap::new();

    for opt in set.iter() {
        if opt.as_ref().match_style(Style::Pos)
            || opt.as_ref().match_style(Style::Cmd)
            || opt.as_ref().match_style(Style::Main)
        {
            let index = opt
                .as_ref()
                .get_index()
                .unwrap()
                .calc_index(MAX_INDEX, 0)
                .unwrap_or(MAX_INDEX);
            let entry = index_map.entry(index).or_insert(vec![]);

            entry.push(opt.as_ref().get_uid());
        }
    }

    let mut names = vec![];

    for (index, uids) in index_map.iter() {
        let valid;

        // <cmd1> <cmd2> <pos3> [pos4] [pos5]
        // any of thing at position 1
        if index == &1 || index == &0 {
            let mut cmd_count = 0;
            let mut cmd_valid = false;
            let mut pos_valid = false;
            let mut value_valid = false;

            for uid in uids {
                let opt = set.get_opt(*uid).unwrap();

                if opt.match_style(Style::Cmd) {
                    cmd_count += 1;
                    // set the cmd will valid the check
                    cmd_valid = cmd_valid || opt.check().unwrap_or(false);
                    // if any of cmd is valid, break out
                    if cmd_valid {
                        break;
                    }
                    names.push(opt.get_hint().to_owned());
                } else if opt.match_style(Style::Pos) {
                    let opt_valid = opt.check().unwrap_or(false);

                    pos_valid = pos_valid || opt_valid;
                    if opt_valid {
                        value_valid = value_valid || opt.has_value();
                        // check if any the POS has a value
                        if value_valid {
                            break;
                        }
                    }
                    names.push(opt.get_hint().to_owned());
                }
            }
            // any of CMD
            // or
            // any of POS set
            if cmd_count > 0 {
                valid = cmd_valid || value_valid;
            } else {
                // any of POS is set
                // or
                // all the POS is optional
                valid = value_valid || pos_valid;
            }
        } else {
            // <pos1> [pos2] [pos3] [pos4] [pos5]
            // if any of POS is force required,
            // must set one POS an current position
            let mut pos_valid = false;
            let mut value_valid = false;

            for uid in uids {
                let opt = set.get_opt(*uid).unwrap();
                let opt_valid = opt.check().unwrap_or(false);

                pos_valid = pos_valid || opt_valid;
                // if POS is invalid, it must be force required but not set by user
                if opt_valid {
                    value_valid = value_valid || opt.has_value();
                    // if any POS is set, break out
                    if value_valid {
                        break;
                    }
                }
                names.push(opt.get_hint().to_owned());
            }
            // any of POS is set
            // or
            // all the POS is optional
            valid = value_valid || pos_valid;
        }
        if !valid {
            return Err(Error::ForceRequiredPostionOption(
                *index,
                names.join(" or "),
            ));
        }
        names.clear();
    }

    Ok(true)
}

pub fn default_post_check<S: Set, P: Parser<S>>(_set: &S, _parser: &P) -> Result<bool> {
    Ok(true)
}
