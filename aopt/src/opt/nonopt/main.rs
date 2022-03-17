use std::convert::{TryFrom, TryInto};
use ustr::Ustr;

use super::NonOpt;
use crate::err::Error;
use crate::gstr;
use crate::opt::*;
use crate::set::CreateInfo;
use crate::set::Creator;
use crate::uid::Uid;

pub trait Main: NonOpt {}

#[derive(Debug)]
pub struct MainOpt {
    uid: Uid,

    name: Ustr,

    value: OptValue,

    need_invoke: bool,

    help_info: HelpInfo,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        unsafe impl Send for MainOpt { }
        unsafe impl Sync for MainOpt { }
    }
}

impl TryFrom<CreateInfo> for MainOpt {
    type Error = Error;

    fn try_from(value: CreateInfo) -> Result<Self> {
        let mut ci = value;
        let help_info = HelpInfo::from(&mut ci);

        Ok(Self {
            uid: ci.get_uid(),
            name: ci.get_name().clone(),
            value: OptValue::Null,
            need_invoke: false,
            help_info,
        })
    }
}

impl Main for MainOpt {}

impl Opt for MainOpt {}

impl NonOpt for MainOpt {}

impl Type for MainOpt {
    fn get_type_name(&self) -> Ustr {
        MainCreator::type_name()
    }

    fn is_deactivate_style(&self) -> bool {
        false
    }

    fn match_style(&self, style: Style) -> bool {
        match style {
            Style::Main => true,
            _ => false,
        }
    }

    fn check(&self) -> Result<()> {
        if !(self.get_optional() || self.has_value()) {
            Err(Error::sp_option_force_require(self.get_hint()))
        } else {
            Ok(())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Identifier for MainOpt {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }
}

impl Callback for MainOpt {
    fn is_need_invoke(&self) -> bool {
        self.need_invoke
    }

    fn set_invoke(&mut self, invoke: bool) {
        self.need_invoke = invoke;
    }

    fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool {
        match callback_type {
            CallbackType::Main | CallbackType::MainMut => true,
            _ => false,
        }
    }

    fn set_callback_ret(&mut self, ret: Option<OptValue>) -> Result<()> {
        if let Some(ret) = ret {
            self.set_value(ret);
        }
        Ok(())
    }
}

impl Name for MainOpt {
    fn get_name(&self) -> Ustr {
        self.name
    }

    fn get_prefix(&self) -> Ustr {
        gstr("")
    }

    fn set_name(&mut self, string: Ustr) {
        self.name = string;
    }

    fn set_prefix(&mut self, _string: Ustr) {}

    fn match_name(&self, _name: Ustr) -> bool {
        true
    }

    fn match_prefix(&self, _prefix: Ustr) -> bool {
        false
    }
}

impl Optional for MainOpt {
    fn get_optional(&self) -> bool {
        true
    }

    fn set_optional(&mut self, _optional: bool) {}

    fn match_optional(&self, optional: bool) -> bool {
        self.get_optional() == optional
    }
}

impl Alias for MainOpt {
    fn get_alias(&self) -> Option<&Vec<(Ustr, Ustr)>> {
        None
    }

    fn add_alias(&mut self, _prefix: Ustr, _name: Ustr) {}

    fn rem_alias(&mut self, _prefix: Ustr, _name: Ustr) {}

    fn match_alias(&self, _prefix: Ustr, _name: Ustr) -> bool {
        false
    }
}

impl Index for MainOpt {
    fn get_index(&self) -> Option<&OptIndex> {
        None
    }

    fn set_index(&mut self, _: OptIndex) {}

    fn match_index(&self, _total: u64, _current: u64) -> bool {
        true
    }
}

impl Value for MainOpt {
    fn get_value(&self) -> &OptValue {
        &self.value
    }

    fn get_value_mut(&mut self) -> &mut OptValue {
        &mut self.value
    }

    fn get_default_value(&self) -> &OptValue {
        &OptValue::Null
    }

    fn set_value(&mut self, value: OptValue) {
        self.value = value;
    }

    fn set_default_value(&mut self, _value: OptValue) {}

    fn parse_value(&self, _string: Ustr, _disable: bool, _index: u64) -> Result<OptValue> {
        Ok(OptValue::from(true))
    }

    fn has_value(&self) -> bool {
        !self.get_value().is_null()
    }

    fn reset_value(&mut self) {
        self.value = self.get_default_value().clone();
    }
}

impl Help for MainOpt {
    fn set_hint(&mut self, hint: Ustr) {
        self.help_info.set_hint(hint);
    }

    fn set_help(&mut self, help: Ustr) {
        self.help_info.set_help(help);
    }

    fn get_help_info(&self) -> &HelpInfo {
        &self.help_info
    }
}

/// [`Creator`] implementation of option type [`MainOpt`].
#[derive(Debug, Default, Clone)]
pub struct MainCreator;

impl MainCreator {
    pub fn type_name() -> Ustr {
        gstr("m")
    }
}

impl Creator for MainCreator {
    fn get_type_name(&self) -> Ustr {
        MainCreator::type_name()
    }

    fn is_support_deactivate_style(&self) -> bool {
        false
    }

    fn create_with(&self, create_info: CreateInfo) -> Result<Box<dyn Opt>> {
        if create_info.get_support_deactivate_style() {
            if !self.is_support_deactivate_style() {
                return Err(Error::opt_unsupport_deactivate_style(
                    create_info.get_name(),
                ));
            }
        }

        assert_eq!(create_info.get_type_name(), self.get_type_name());

        let opt: MainOpt = create_info.try_into()?;

        trace!(?opt, "create a Main");
        Ok(Box::new(opt))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_type_main_work() {
        let creator = MainCreator::default();

        assert_eq!(creator.get_type_name(), MainCreator::type_name());
        // main not support deactivate style
        assert_eq!(creator.is_support_deactivate_style(), false);

        let mut ci = CreateInfo::parse(gstr("main=m"), &[]).unwrap();

        ci.set_uid(1);

        let mut main = creator.create_with(ci).unwrap();

        assert_eq!(main.get_type_name(), MainCreator::type_name());
        assert_eq!(main.is_deactivate_style(), false);
        assert_eq!(main.match_style(Style::Main), true);
        assert_eq!(main.check().is_err(), false);

        assert_eq!(main.get_uid(), 1);
        main.set_uid(42);
        assert_eq!(main.get_uid(), 42);

        assert_eq!(main.is_need_invoke(), false);
        main.set_invoke(true);
        assert_eq!(main.is_need_invoke(), true);
        assert_eq!(main.is_accept_callback_type(CallbackType::Main), true);
        assert_eq!(main.is_accept_callback_type(CallbackType::MainMut), true);

        // main not support alias
        main.add_alias("-".into(), "m".into());
        assert_eq!(main.get_alias(), None);
        assert_eq!(main.match_alias("-".into(), "m".into()), false);
        main.rem_alias("-".into(), "m".into());
        assert_eq!(main.get_alias(), None);

        assert_eq!(main.get_index(), None);
        assert_eq!(main.match_index(6, 1), true);
        assert_eq!(main.match_index(6, 3), true);
        main.set_index(OptIndex::forward(1));
        assert_eq!(main.get_index(), None);
        assert_eq!(main.match_index(6, 9), true);

        assert_eq!(main.get_name(), gstr("main"));
        assert_eq!(main.get_prefix(), gstr(""));
        assert_eq!(main.match_name("www".into()), true);
        assert_eq!(main.match_name("main".into()), true);
        assert_eq!(main.match_prefix("--".into()), false);
        assert_eq!(main.match_prefix("".into()), false);
        main.set_name(gstr("main1"));
        main.set_prefix(gstr("+"));
        assert_eq!(main.match_name("www".into()), true);
        assert_eq!(main.match_name("main1".into()), true);
        assert_eq!(main.get_name(), "main1");
        assert_eq!(main.match_prefix("+".into()), false);
        assert_eq!(main.match_prefix("".into()), false);

        assert_eq!(main.get_optional(), true);
        assert_eq!(main.match_optional(true), true);
        main.set_optional(false);
        assert_eq!(main.get_optional(), true);
        assert_eq!(main.match_optional(true), true);
        assert_eq!(main.check().is_err(), false);

        assert_eq!(main.get_value().is_null(), true);
        assert_eq!(main.get_default_value().is_null(), true);
        assert_eq!(main.has_value(), false);
        let value = main.parse_value("".into(), false, 0);
        assert_eq!(value.is_ok(), true);
        let value = value.unwrap();
        assert_eq!(value.is_bool(), true);
        main.set_value(value);
        assert_eq!(main.get_value().as_bool(), OptValue::from(true).as_bool());
        main.set_default_value(OptValue::from(false));
        assert_eq!(main.get_default_value().is_null(), true);
        main.reset_value();
        assert_eq!(main.get_value().is_null(), true);

        assert_eq!(main.as_ref().as_any().is::<MainOpt>(), true);
    }
}
