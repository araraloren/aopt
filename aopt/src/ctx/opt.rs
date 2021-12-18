use ustr::Ustr;

use super::Context;

use crate::err::Error;
use crate::err::Result;
use crate::gstr;
use crate::opt::Opt;
use crate::opt::OptValue;
use crate::opt::Style;
use crate::uid::Uid;

#[derive(Debug)]
pub struct OptContext {
    prefix: Ustr,

    name: Ustr,

    argument: Option<Ustr>,

    style: Style,

    consume_arg: bool,

    value: Option<OptValue>,

    matched_uid: Option<Uid>,

    disable: bool,
}

impl OptContext {
    pub fn new(
        prefix: Ustr,
        name: Ustr,
        argument: Option<Ustr>,
        style: Style,
        consume_arg: bool,
        disable: bool,
    ) -> Self {
        Self {
            prefix,
            name,
            argument,
            style,
            consume_arg,
            value: None,
            matched_uid: None,
            disable,
        }
    }
}

impl Context for OptContext {
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        let mut matched = opt.match_style(self.style);

        if matched {
            matched = matched
                && ((opt.match_name(self.name) && opt.match_prefix(self.prefix))
                    || opt.match_alias(self.prefix, self.name));
        }
        info!(%matched, "Matching context with opt<{}>", opt.get_uid());
        trace!(?self, ?opt, "matching ...");
        if matched {
            if self.is_comsume_argument() && self.argument.is_none() {
                return Err(Error::sp_missing_argument(opt.get_hint()));
            }
            if !opt.is_deactivate_style() && self.disable {
                return Err(Error::sp_unsupport_deactivate_style(opt.get_hint()));
            } else {
                let value = opt
                    .parse_value(self.argument.unwrap_or(gstr("")))
                    .map_err(|_| Error::sp_invalid_argument(opt.get_hint()))?;
                self.set_value(value);
                debug!("get return and will set value {:?}!", self.get_value());
            }
            opt.set_invoke(true);
            self.matched_uid = Some(opt.get_uid());
        }
        Ok(matched)
    }

    fn undo(&mut self, opt: &mut dyn Opt) {
        self.value = None;
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

    fn get_style(&self) -> Style {
        self.style
    }

    fn get_next_argument(&self) -> &Option<Ustr> {
        &self.argument
    }

    fn is_comsume_argument(&self) -> bool {
        self.consume_arg
    }
}
