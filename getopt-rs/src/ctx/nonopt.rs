use std::borrow::Cow;

use super::Context;
use crate::err::Result;
use crate::opt::{Opt, Style};
use crate::uid::Uid;

#[derive(Debug)]
pub struct NonOptContext<'name> {
    uid: Uid,

    name: Cow<'name, str>,

    style: Style,

    total: u64,

    current: u64,

    matched_index: Option<usize>,
}

impl<'name> NonOptContext<'name> {
    pub fn new(uid: Uid, name: Cow<'name, str>, style: Style, total: u64, current: u64) -> Self {
        Self {
            uid,
            name,
            style,
            total,
            current,
            matched_index: None,
        }
    }
}

impl<'name> Context for NonOptContext<'name> {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn match_opt(&self, opt: &dyn Opt) -> bool {
        let mut matched = opt.match_style(self.style);
        debug!(
            "Matching option<{}> <-> nonopt context<{}>",
            opt.get_uid(),
            self.get_uid()
        );
        if matched {
            matched = matched
                && (opt.match_name(self.name.as_ref())
                    && opt.match_index(self.total, self.current));
        }
        debug!(">>>> {}", if matched { "TRUE" } else { "FALSE" });
        matched
    }

    fn process_opt(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        self.matched_index = Some(self.current as usize);
        debug!("Set data of option<{}>", opt.get_uid());
        // try to set value here happy some check
        // in parser, we will set the value to return value of callback
        opt.set_value(opt.parse_value("")?);
        opt.set_invoke(true);
        Ok(true)
    }

    fn get_matched_index(&self) -> Option<usize> {
        self.matched_index
    }

    fn get_style(&self) -> Style {
        self.style
    }

    fn get_next_argument(&self) -> &Option<String> {
        &None
    }

    fn is_comsume_argument(&self) -> bool {
        false
    }
}
