use ustr::Ustr;

use super::Context;

use crate::err::Error;
use crate::err::Result;
use crate::opt::Opt;
use crate::opt::OptValue;
use crate::opt::Style;
use crate::uid::Uid;

/// The [`Context`] using for matching [`NonOpt`](crate::opt::NonOpt).
#[derive(Debug)]
pub struct NonOptContext {
    name: Ustr,

    style: Style,

    total: usize,

    index: usize,

    value: Option<OptValue>,

    matched_index: Option<usize>,

    matched_uid: Option<Uid>,
}

impl NonOptContext {
    pub fn new(name: Ustr, style: Style, index: usize, total: usize) -> Self {
        Self {
            name,
            style,
            total,
            index,
            value: None,
            matched_index: None,
            matched_uid: None,
        }
    }
}

impl Context for NonOptContext {
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        // 1. matching the option style.
        let mut matched = opt.match_style(self.style);

        if matched {
            // 2. matching the option name only.
            // 3. matching the option index.
            matched =
                matched && (opt.match_name(self.name) && opt.match_index(self.index, self.total));
        }
        info!(%matched, "Matching context with non-opt<{}>", opt.get_uid());
        trace!(?self, ?opt, "matching ...");
        if matched {
            let value = opt
                .parse_value(self.name, false, self.index, self.total)
                .map_err(|_| Error::sp_invalid_argument(opt.get_hint()))?;
            // 4. call the Opt::parse_value generate and set the value.
            self.set_value(value);
            debug!("get return value {:?}!", self.get_value());
            // 5. set the invoke flag.
            opt.set_invoke(true);
            self.matched_uid = Some(opt.get_uid());
            self.matched_index = Some(self.index as usize);
        }
        Ok(matched)
    }

    fn undo(&mut self, opt: &mut dyn Opt) {
        self.value = None;
        self.matched_index = None;
        self.matched_uid = None;
        opt.set_invoke(false);
    }

    fn get_value(&self) -> Option<&OptValue> {
        self.value.as_ref()
    }

    fn take_value(&mut self) -> Option<OptValue> {
        self.value.take()
    }

    fn set_value(&mut self, value: OptValue) {
        self.value = Some(value);
    }

    fn get_matched_uid(&self) -> Option<Uid> {
        self.matched_uid
    }

    fn set_matched_uid(&mut self, uid: Option<Uid>) {
        self.matched_uid = uid;
    }

    fn get_matched_index(&self) -> Option<usize> {
        self.matched_index
    }

    fn set_matched_index(&mut self, index: Option<usize>) {
        self.matched_index = index;
    }

    fn get_style(&self) -> Style {
        self.style
    }

    fn get_argument(&self) -> &Option<Ustr> {
        &None
    }

    fn is_comsume_argument(&self) -> bool {
        false
    }

    fn is_matched(&self) -> bool {
        self.matched_uid.is_some() && self.matched_index.is_some()
    }
}
