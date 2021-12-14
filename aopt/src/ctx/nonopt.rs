use ustr::Ustr;

use super::Context;

use crate::err::Error;
use crate::err::Result;
use crate::opt::Opt;
use crate::opt::OptValue;
use crate::opt::Style;

#[derive(Debug)]
pub struct NonOptContext {
    name: Ustr,

    style: Style,

    total: u64,

    current: u64,

    value: Option<OptValue>,

    matched_index: Option<usize>,
}

impl NonOptContext {
    pub fn new(name: Ustr, style: Style, total: u64, current: u64) -> Self {
        Self {
            name,
            style,
            total,
            current,
            value: None,
            matched_index: None,
        }
    }
}

impl Context for NonOptContext {
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        let mut matched = opt.match_style(self.style);

        if matched {
            matched =
                matched && (opt.match_name(self.name) && opt.match_index(self.total, self.current));
        }
        info!(%matched, "Matching context with non-opt<{}>", opt.get_uid());
        trace!(?self, ?opt, "matching ...");
        if matched {
            self.matched_index = Some(self.current as usize);
            let value = opt
                .parse_value(self.name)
                .map_err(|_| Error::sp_invalid_argument(opt.get_hint()))?;
            self.set_value(value);
            debug!("get return value {:?}!", self.get_value());
            opt.set_invoke(true);
        }
        Ok(matched)
    }

    fn undo(&mut self) {
        self.value = None;
        self.matched_index = None;
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

    fn get_matched_index(&self) -> Option<usize> {
        self.matched_index
    }

    fn set_matched_index(&mut self, index: Option<usize>) {
        self.matched_index = index;
    }

    fn get_style(&self) -> Style {
        self.style
    }

    fn get_next_argument(&self) -> &Option<Ustr> {
        &None
    }

    fn is_comsume_argument(&self) -> bool {
        false
    }
}
