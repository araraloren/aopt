use super::Context;
use crate::err::{Error, Result};
use crate::opt::{Opt, OptValue, Style};

#[derive(Debug)]
pub struct OptContext {
    prefix: String,

    name: String,

    argument: Option<String>,

    style: Style,

    consume_arg: bool,

    value: Option<OptValue>,

    matched_index: Option<usize>,
}

impl OptContext {
    pub fn new(
        prefix: String,
        name: String,
        argument: Option<String>,
        style: Style,
        consume_arg: bool,
    ) -> Self {
        Self {
            prefix,
            name,
            argument,
            style,
            consume_arg,
            value: None,
            matched_index: None,
        }
    }
}

impl Context for OptContext {
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        let mut matched = opt.match_style(self.style);

        debug!(
            "Matching option<{}> <-> opt context<{:?}>",
            opt.get_uid(),
            self
        );
        if matched {
            matched = matched
                && ((opt.match_name(self.name.as_ref()) && opt.match_prefix(self.prefix.as_ref()))
                    || opt.match_alias(self.prefix.as_ref(), self.name.as_ref()));
        }
        debug!(">>>> {}", if matched { "TRUE" } else { "FALSE" });
        if matched {
            if self.is_comsume_argument() && self.argument.is_none() {
                return Err(Error::RequiredArgumentOfOption(String::from(
                    opt.get_hint(),
                )));
            }
            self.matched_index = Some(0);
            self.set_value(opt.parse_value(self.argument.as_ref().unwrap().as_str())?);
            debug!(
                "Keep value of option<{}> ==> {:?}",
                opt.get_uid(),
                self.get_value()
            );
            opt.set_invoke(true);
        }
        Ok(matched)
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

    fn get_next_argument(&self) -> &Option<String> {
        &self.argument
    }

    fn is_comsume_argument(&self) -> bool {
        self.consume_arg
    }
}
