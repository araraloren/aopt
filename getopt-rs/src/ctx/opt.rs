use super::Context;
use crate::err::{Error, Result};
use crate::opt::{Opt, Style};

#[derive(Debug)]
pub struct OptContext {
    prefix: String,

    name: String,

    argument: Option<String>,

    style: Style,

    consume_arg: bool,

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
            matched_index: None,
        }
    }
}

impl Context for OptContext {
    fn match_opt(&self, opt: &dyn Opt) -> bool {
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
        let mut value = &String::default();
        if let Some(v) = &self.argument {
            value = v;
        }
        opt.set_value(opt.parse_value(value.as_str())?);
        opt.set_invoke(true);
        debug!("after setting ==> {:?}", opt);
        Ok(true)
    }

    fn get_matched_index(&self) -> Option<usize> {
        self.matched_index
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
