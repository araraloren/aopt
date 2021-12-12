use ustr::Ustr;

use super::Context;

use crate::err::Result;
use crate::err::SpecialError;
use crate::gstr;
use crate::opt::Opt;
use crate::opt::OptValue;
use crate::opt::Style;

#[derive(Debug)]
pub struct OptContext {
    prefix: Ustr,

    name: Ustr,

    argument: Option<Ustr>,

    style: Style,

    consume_arg: bool,

    value: Option<OptValue>,

    matched_index: Option<usize>,

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
            matched_index: None,
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
                return Err(
                    SpecialError::MissingArgumentForOption(opt.get_hint().to_owned()).into(),
                );
            }
            self.matched_index = Some(0);
            if !opt.is_deactivate_style() && self.disable {
                return Err(SpecialError::CanNotDisableOption(format!(
                    "{}",
                    opt.get_name().as_ref()
                ))
                .into());
            } else {
                let value = opt
                    .parse_value(self.argument.unwrap_or(gstr("")))
                    .map_err(|_| {
                        SpecialError::InvalidArgumentForOption(opt.get_hint().to_owned())
                    })?;
                self.set_value(value);
                debug!("get return and will set value {:?}!", self.get_value());
            }
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
        &self.argument
    }

    fn is_comsume_argument(&self) -> bool {
        self.consume_arg
    }
}
