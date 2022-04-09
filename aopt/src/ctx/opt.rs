use ustr::Ustr;

use super::Context;

use crate::err::Error;
use crate::err::Result;
use crate::gstr;
use crate::opt::Opt;
use crate::opt::OptValue;
use crate::opt::Style;
use crate::uid::Uid;

/// The [`Context`] using for matching [`Opt`].
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

    index: usize,

    total: usize,
}

impl OptContext {
    pub fn new(
        prefix: Ustr,
        name: Ustr,
        argument: Option<Ustr>,
        style: Style,
        consume_arg: bool,
        disable: bool,
        index: usize,
        total: usize,
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
            index,
            total,
        }
    }
}

impl Context for OptContext {
    fn process(&mut self, opt: &mut dyn Opt) -> Result<bool> {
        // 1. matching the option style.
        let mut matched = opt.match_style(self.style);

        if matched {
            // 2. matching the option name and prefix
            // 3. check if any alias matching current name and prefix.
            matched = matched
                && ((opt.match_name(self.name) && opt.match_prefix(self.prefix))
                    || opt.match_alias(self.prefix, self.name));
        }
        info!(%matched, "Matching context with opt<{}>", opt.get_uid());
        trace!(?self, ?opt, "matching ...");
        if matched {
            // 4. check if we need an argument for the `Opt`.
            if self.is_comsume_argument() && self.argument.is_none() {
                return Err(Error::sp_missing_argument(opt.get_hint()));
            }
            // 5. parsing the value
            let value = opt
                .parse_value(
                    self.argument.unwrap_or_else(|| gstr("")),
                    self.disable,
                    self.index,
                    self.total,
                )
                .map_err(|_| Error::sp_invalid_argument(opt.get_hint()))?;
            // 6. call the Opt::parse_value generate and set the value.
            self.set_value(value);
            debug!("get return and will set value {:?}!", self.get_value());
            // 7. set the invoke flag.
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

    fn get_argument(&self) -> &Option<Ustr> {
        &self.argument
    }

    fn is_comsume_argument(&self) -> bool {
        self.consume_arg
    }
}
