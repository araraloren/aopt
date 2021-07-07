use std::mem::take;

use super::NonOpt;
use crate::opt::*;
use crate::set::CreateInfo;
use crate::set::Creator;
use crate::uid::Uid;

pub fn current_type() -> &'static str {
    "p"
}

pub trait Pos: NonOpt {}

#[derive(Debug)]
pub struct PosOpt {
    uid: Uid,

    name: String,

    prefix: String,

    optional: bool,

    value: OptValue,

    index: OptIndex,

    need_invoke: bool,

    help_info: HelpInfo,
}

impl From<CreateInfo> for PosOpt {
    fn from(ci: CreateInfo) -> Self {
        let mut ci = ci;

        Self {
            uid: ci.get_uid(),
            name: take(ci.get_name_mut()),
            prefix: String::from(""),
            optional: ci.get_optional(),
            value: OptValue::from(false),
            index: take(ci.get_index_mut()),
            need_invoke: false,
            help_info: take(ci.get_help_info_mut()),
        }
    }
}

impl Pos for PosOpt {}

impl Opt for PosOpt {}

impl NonOpt for PosOpt {}

impl Type for PosOpt {
    fn get_type_name(&self) -> &'static str {
        current_type()
    }

    fn is_deactivate_style(&self) -> bool {
        false
    }

    fn match_style(&self, style: Style) -> bool {
        match style {
            Style::Pos => true,
            _ => false,
        }
    }

    fn check(&self) -> Result<bool> {
        if !(self.get_optional() || self.has_value()) {
            Err(Error::ForceRequiredOption(self.get_hint().to_owned()))
        } else {
            Ok(true)
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Identifier for PosOpt {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }
}

impl Callback for PosOpt {
    fn is_need_invoke(&self) -> bool {
        self.need_invoke
    }

    fn set_invoke(&mut self, invoke: bool) {
        self.need_invoke = invoke;
    }

    fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool {
        if self.is_need_invoke() {
            match callback_type {
                CallbackType::Pos | CallbackType::PosMut => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn set_callback_ret(&mut self, ret: Option<OptValue>) -> Result<()> {
        if let Some(ret) = ret {
            self.set_value(ret);
        }
        Ok(())
    }
}

impl Name for PosOpt {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_prefix(&self) -> &str {
        ""
    }

    fn set_name(&mut self, string: String) {
        self.name = string;
    }

    fn set_prefix(&mut self, _string: String) {}

    fn match_name(&self, _name: &str) -> bool {
        true
    }

    fn match_prefix(&self, _prefix: &str) -> bool {
        false
    }
}

impl Optional for PosOpt {
    fn get_optional(&self) -> bool {
        self.optional
    }

    fn set_optional(&mut self, optional: bool) {
        self.optional = optional;
    }

    fn match_optional(&self, optional: bool) -> bool {
        self.get_optional() == optional
    }
}

impl Alias for PosOpt {
    fn get_alias(&self) -> Option<&Vec<(String, String)>> {
        None
    }

    fn add_alias(&mut self, _prefix: String, _name: String) {}

    fn rem_alias(&mut self, _prefix: &str, _name: &str) {}

    fn match_alias(&self, _prefix: &str, _name: &str) -> bool {
        false
    }
}

impl Index for PosOpt {
    fn get_index(&self) -> Option<&OptIndex> {
        Some(&self.index)
    }

    fn set_index(&mut self, index: OptIndex) {
        self.index = index;
    }

    fn match_index(&self, total: u64, current: u64) -> bool {
        match self.get_index() {
            Some(realindex) => match realindex.calc_index(total, current) {
                Some(realindex) => return realindex == current,
                None => {}
            },
            None => {}
        }
        false
    }
}

impl Value for PosOpt {
    fn get_value(&self) -> &OptValue {
        &self.value
    }

    fn get_default_value(&self) -> &OptValue {
        &OptValue::Null
    }

    fn set_value(&mut self, value: OptValue) {
        self.value = value;
    }

    fn set_default_value(&mut self, _value: OptValue) {}

    fn parse_value(&self, _string: &str) -> Result<OptValue> {
        Ok(OptValue::from(true))
    }

    fn has_value(&self) -> bool {
        !self.get_value().is_null()
    }

    fn reset_value(&mut self) {
        self.value = OptValue::from(false);
    }
}

impl Help for PosOpt {
    fn set_hint(&mut self, hint: String) {
        self.help_info.set_hint(hint);
    }

    fn set_help(&mut self, help: String) {
        self.help_info.set_help(help);
    }

    fn get_help_info(&self) -> &HelpInfo {
        &self.help_info
    }
}

#[derive(Debug, Default, Clone)]
pub struct PosCreator;

impl Creator for PosCreator {
    fn get_type_name(&self) -> &'static str {
        current_type()
    }

    fn is_support_deactivate_style(&self) -> bool {
        false
    }

    fn create_with(&self, create_info: CreateInfo) -> Result<Box<dyn Opt>> {
        if create_info.get_support_deactivate_style() {
            if !self.is_support_deactivate_style() {
                return Err(Error::NotSupportDeactivateStyle(
                    create_info.get_name().to_owned(),
                ));
            }
        }
        if create_info.get_index().is_null() {
            return Err(Error::ForceRequiredOptionIndex(
                create_info.get_name().to_owned(),
            ));
        }

        assert_eq!(create_info.get_type_name(), self.get_type_name());

        let opt: PosOpt = create_info.into();

        Ok(Box::new(opt))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_type_pos_work() {
        let creator = PosCreator::default();

        assert_eq!(creator.get_type_name(), current_type());
        // pos not support deactivate style
        assert_eq!(creator.is_support_deactivate_style(), false);

        let mut ci = CreateInfo::parse("pos=p@1", &[]).unwrap();

        ci.set_uid(1);

        let mut pos = creator.create_with(ci).unwrap();

        assert_eq!(pos.get_type_name(), current_type());
        assert_eq!(pos.is_deactivate_style(), false);
        assert_eq!(pos.match_style(Style::Pos), true);
        assert_eq!(pos.check().is_err(), false);

        assert_eq!(pos.get_uid(), 1);
        pos.set_uid(42);
        assert_eq!(pos.get_uid(), 42);

        assert_eq!(pos.is_need_invoke(), false);
        pos.set_invoke(true);
        assert_eq!(pos.is_need_invoke(), true);
        assert_eq!(pos.is_accept_callback_type(CallbackType::Pos), true);
        assert_eq!(pos.is_accept_callback_type(CallbackType::PosMut), true);

        // pos not support alias
        pos.add_alias("-".to_owned(), "m".to_owned());
        assert_eq!(pos.get_alias(), None);
        assert_eq!(pos.match_alias("-", "m"), false);
        pos.rem_alias("-", "m");
        assert_eq!(pos.get_alias(), None);

        assert_eq!(pos.get_index(), Some(&OptIndex::forward(1)));
        assert_eq!(pos.match_index(6, 1), true);
        assert_eq!(pos.match_index(6, 2), false);
        pos.set_index(OptIndex::forward(3));
        assert_eq!(pos.match_index(6, 1), false);
        assert_eq!(pos.match_index(6, 3), true);
        assert_eq!(pos.get_index(), Some(&OptIndex::forward(3)));
        assert_eq!(pos.match_index(6, 9), false);

        assert_eq!(pos.get_name(), "pos");
        assert_eq!(pos.get_prefix(), "");
        assert_eq!(pos.match_name("www"), true);
        assert_eq!(pos.match_name("pos"), true);
        assert_eq!(pos.match_prefix("--"), false);
        assert_eq!(pos.match_prefix(""), false);
        pos.set_name(String::from("pos1"));
        pos.set_prefix(String::from("+"));
        assert_eq!(pos.match_name("www"), true);
        assert_eq!(pos.match_name("pos1"), true);
        assert_eq!(pos.get_name(), "pos1");
        assert_eq!(pos.match_prefix("+"), false);
        assert_eq!(pos.match_prefix(""), false);

        assert_eq!(pos.get_optional(), true);
        assert_eq!(pos.match_optional(true), true);
        pos.set_optional(false);
        assert_eq!(pos.get_optional(), false);
        assert_eq!(pos.match_optional(true), false);
        assert_eq!(pos.check().is_err(), true);

        assert_eq!(pos.get_value().as_bool(), OptValue::from(false).as_bool());
        assert_eq!(pos.get_default_value().is_null(), true);
        assert_eq!(pos.has_value(), false);
        let value = pos.parse_value("");
        assert_eq!(value.is_ok(), true);
        let value = value.unwrap();
        assert_eq!(value.is_bool(), true);
        pos.set_value(value);
        assert_eq!(pos.get_value().as_bool(), OptValue::from(true).as_bool());
        pos.set_default_value(OptValue::from(false));
        assert_eq!(pos.get_default_value().is_null(), true);
        pos.reset_value();
        assert_eq!(pos.get_value().as_bool(), OptValue::from(false).as_bool());

        assert_eq!(pos.as_ref().as_any().is::<PosOpt>(), true);
    }
}
