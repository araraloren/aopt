use std::mem::take;

use super::help::HelpInfo;
use super::index::Index as OptIndex;
use super::style::Style;
use super::value::Value as OptValue;
use super::*;
use crate::set::info::CreateInfo;
use crate::set::Creator;
use crate::uid::Uid;

pub fn current_type() -> &'static str {
    "b"
}

pub trait Bool: Opt {}

#[derive(Debug)]
pub struct BoolOpt {
    uid: Uid,

    name: String,

    prefix: String,

    optional: bool,

    value: OptValue,

    default_value: OptValue,

    deactivate_style: bool,

    alias: Vec<(String, String)>,

    callback_type: CallbackType,

    help_info: HelpInfo,
}

impl From<CreateInfo> for BoolOpt {
    fn from(ci: CreateInfo) -> Self {
        let mut ci = ci;

        Self {
            uid: ci.get_uid(),
            name: take(ci.get_name_mut()),
            prefix: take(ci.get_prefix_mut()).unwrap(),
            optional: ci.get_optional(),
            value: OptValue::default(),
            default_value: take(ci.get_default_value_mut()),
            deactivate_style: ci.get_support_deactivate_style(),
            alias: take(ci.get_alias_mut()),
            callback_type: ci.get_callback_type().clone(),
            help_info: take(ci.get_help_info_mut()),
        }
    }
}

impl Bool for BoolOpt {}

impl Opt for BoolOpt {}

impl Type for BoolOpt {
    fn get_type_name(&self) -> &'static str {
        current_type()
    }

    fn is_deactivate_style(&self) -> bool {
        self.deactivate_style
    }

    fn match_style(&self, style: Style) -> bool {
        match style {
            Style::Boolean | Style::Multiple => true,
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

impl Identifier for BoolOpt {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }
}

impl Callback for BoolOpt {
    fn get_callback_type(&self) -> &CallbackType {
        &self.callback_type
    }

    fn set_callback_type(&mut self, callback_type: CallbackType) {
        self.callback_type = callback_type;
    }

    fn is_need_invoke(&self) -> bool {
        !self.callback_type.is_null()
    }

    fn set_invoke(&mut self, invoke: bool, mutbale: bool) {
        self.callback_type = if invoke {
            if mutbale {
                CallbackType::Opt
            } else {
                CallbackType::OptMut
            }
        } else {
            CallbackType::default()
        }
    }

    fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool {
        match callback_type {
            CallbackType::Opt | CallbackType::OptMut => true,
            _ => false,
        }
    }
}

impl Name for BoolOpt {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn set_name(&mut self, string: String) {
        self.name = string;
    }

    fn set_prefix(&mut self, string: String) {
        self.prefix = string;
    }

    fn match_name(&self, name: &str) -> bool {
        self.get_name() == name
    }

    fn match_prefix(&self, prefix: &str) -> bool {
        self.get_prefix() == prefix
    }
}

impl Optional for BoolOpt {
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

impl Alias for BoolOpt {
    fn get_alias(&self) -> Option<&Vec<(String, String)>> {
        Some(&self.alias)
    }

    fn add_alias(&mut self, prefix: String, name: String) {
        self.alias.push((prefix, name));
    }

    fn rem_alias(&mut self, prefix: &str, name: &str) {
        for (index, value) in self.alias.iter().enumerate() {
            if value.0 == prefix && value.1 == name {
                self.alias.remove(index);
                break;
            }
        }
    }

    fn match_alias(&self, prefix: &str, name: &str) -> bool {
        self.alias
            .iter()
            .find(|&v| v.0 == prefix && v.1 == name)
            .is_some()
    }
}

impl Index for BoolOpt {
    fn get_index(&self) -> Option<&OptIndex> {
        None
    }

    fn set_index(&mut self, _index: OptIndex) {
        // option can set anywhere
    }

    fn match_index(&self, _total: u64, _current: u64) -> bool {
        true
    }
}

impl Value for BoolOpt {
    fn get_value(&self) -> &OptValue {
        &self.value
    }

    fn get_default_value(&self) -> &OptValue {
        &self.default_value
    }

    fn set_value(&mut self, value: OptValue) {
        self.value = value;
    }

    fn set_default_value(&mut self, value: OptValue) {
        self.default_value = value;
    }

    fn parse_value(&self, _string: &str) -> Result<OptValue> {
        Ok(OptValue::from(!self.is_deactivate_style()))
    }

    fn has_value(&self) -> bool {
        self.get_value().as_bool() != self.get_default_value().as_bool()
    }

    fn reset_value(&mut self) {
        self.value.reset();
    }
}

impl Help for BoolOpt {
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
pub struct BoolCreator;

impl Creator for BoolCreator {
    fn get_type_name(&self) -> &'static str {
        current_type()
    }

    fn is_support_deactivate_style(&self) -> bool {
        true
    }

    fn create_with(&self, create_info: CreateInfo) -> Result<Box<dyn Opt>> {
        if create_info.get_support_deactivate_style() {
            if !self.is_support_deactivate_style() {
                return Err(Error::NotSupportDeactivateStyle(
                    create_info.get_name().to_owned(),
                ));
            }
        }
        if create_info.get_prefix().is_none() {
            return Err(Error::NeedValidPrefix(current_type()));
        }

        assert_eq!(create_info.get_type_name(), self.get_type_name());

        let opt: BoolOpt = create_info.into();

        Ok(Box::new(opt))
    }
}
