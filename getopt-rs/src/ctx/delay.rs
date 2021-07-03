use std::borrow::Cow;

use super::Context;
use crate::err::{Error, Result};
use crate::opt::{Opt, Style};
use crate::uid::Uid;

#[derive(Debug)]
pub struct DelayContext<'arg, 'pre, 'name> {
    uid: Uid,

    prefix: Cow<'pre, str>,

    name: Cow<'name, str>,

    argument: &'arg Option<String>,

    style: Style,

    consume_arg: bool,

    matched_index: Option<usize>,
}

impl<'arg, 'pre, 'name> DelayContext<'arg, 'pre, 'name> {
    pub fn new(
        uid: Uid,
        prefix: Cow<'pre, str>,
        name: Cow<'name, str>,
        argument: &'arg Option<String>,
        style: Style,
        consume_arg: bool,
    ) -> Self {
        Self {
            uid,
            prefix,
            name,
            argument,
            style,
            consume_arg,
            matched_index: None,
        }
    }
}

impl<'arg, 'pre, 'name> Context for DelayContext<'arg, 'pre, 'name> {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn match_opt(&self, opt: &dyn Opt) -> bool {
        let mut matched = opt.match_style(self.style);

        debug!(
            "Matching option<{}> <-> opt context<{}>",
            opt.get_uid(),
            self.get_uid()
        );
        if matched {
            matched = matched
                && ((opt.match_name(self.name.as_ref()) && opt.match_prefix(self.prefix.as_ref()))
                    || opt.match_alias(self.prefix.as_ref(), self.name.as_ref()));
        }
        debug!(">>>> {}", if matched { "TRUE" } else { "FALSE" });
        matched
    }

    fn process_opt(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        debug!("Set the data of option<{}>", opt.get_uid());
        if self.is_comsume_argument() && self.argument.is_none() {
            return Err(Error::RequiredArgumentOfOption(String::from(
                opt.get_hint(),
            )));
        }
        self.matched_index = Some(0);
        if let Some(_value) = self.argument {
            // don't set value here, because this is using for a delay parse
            // opt.set_value(opt.parse_value(value)?);
            opt.set_invoke(true);
        }
        Ok(true)
    }

    fn get_matched_index(&self) -> Option<usize> {
        self.matched_index
    }

    fn get_style(&self) -> Style {
        self.style
    }

    fn get_next_argument(&self) -> &Option<String> {
        self.argument
    }

    fn is_comsume_argument(&self) -> bool {
        self.consume_arg
    }
}
