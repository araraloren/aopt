use super::Context;
use crate::err::{Result, SpecialError};
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
    #[tracing::instrument]
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        let mut matched = opt.match_style(self.style);

        if matched {
            matched = matched
                && ((opt.match_name(self.name.as_ref()) && opt.match_prefix(self.prefix.as_ref()))
                    || opt.match_alias(self.prefix.as_ref(), self.name.as_ref()));
        }
        info!(%matched, "Matching context with opt<{}>", opt.get_uid());
        trace!(?self, ?opt, "matching ...");
        if matched {
            if self.is_comsume_argument() && self.argument.is_none() {
                return Err(
                    SpecialError::MissingArgumentForOption(opt.get_hint().to_owned()).into(),
                );
            }
            self.matched_index = Some(0);
            if let Some(arg) = self.argument.as_ref() {
                let value = opt.parse_value(arg.as_str()).map_err(|v| {
                    SpecialError::InvalidArgumentForOption(opt.get_hint().to_owned())
                })?;
                self.set_value(value);
            }
            debug!(" get return value {:?}!", self.get_value());
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
