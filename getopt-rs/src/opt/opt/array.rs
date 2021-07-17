use std::mem::take;

use crate::opt::*;
use crate::set::CreateInfo;
use crate::set::Creator;
use crate::uid::Uid;

pub fn current_type() -> &'static str {
    "a"
}

pub trait Array: Opt {}

#[derive(Debug)]
pub struct ArrayOpt {
    uid: Uid,

    name: String,

    prefix: String,

    optional: bool,

    value: OptValue,

    default_value: OptValue,

    alias: Vec<(String, String)>,

    need_invoke: bool,

    help_info: HelpInfo,
}

impl From<CreateInfo> for ArrayOpt {
    fn from(ci: CreateInfo) -> Self {
        let mut ci = ci;
        let help_info = HelpInfo::from(&mut ci);

        Self {
            uid: ci.get_uid(),
            name: take(ci.get_name_mut()),
            prefix: take(ci.get_prefix_mut()).unwrap(),
            optional: ci.get_optional(),
            value: OptValue::default(),
            default_value: take(ci.get_default_value_mut()),
            alias: take(ci.get_alias_mut()),
            need_invoke: false,
            help_info,
        }
    }
}

impl Array for ArrayOpt {}

impl Opt for ArrayOpt {}

impl Type for ArrayOpt {
    fn get_type_name(&self) -> &'static str {
        current_type()
    }

    fn is_deactivate_style(&self) -> bool {
        false
    }

    fn match_style(&self, style: Style) -> bool {
        match style {
            Style::Argument => true,
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

impl Identifier for ArrayOpt {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }
}

impl Callback for ArrayOpt {
    fn is_need_invoke(&self) -> bool {
        self.need_invoke
    }

    fn set_invoke(&mut self, invoke: bool) {
        self.need_invoke = invoke;
    }

    fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool {
        match callback_type {
            CallbackType::Opt | CallbackType::OptMut => true,
            _ => false,
        }
    }

    fn set_callback_ret(&mut self, ret: Option<OptValue>) -> Result<()> {
        if let Some(mut ret) = ret {
            if !ret.is_vec() {
                return Err(Error::InvalidReturnValueOfCallback(
                    "OptValue::Vec".to_owned(),
                    format!("{:?}", ret),
                ));
            }
            // merge the value of return and self
            let mut value = take(&mut self.value);
            if !value.is_vec() {
                value = OptValue::from(vec![]);
            }
            value
                .as_vec_mut()
                .unwrap()
                .append(ret.as_vec_mut().unwrap());
            self.set_value(value);
        }
        Ok(())
    }
}

impl Name for ArrayOpt {
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

impl Optional for ArrayOpt {
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

impl Alias for ArrayOpt {
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

impl Index for ArrayOpt {
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

impl Value for ArrayOpt {
    fn get_value(&self) -> &OptValue {
        &self.value
    }

    fn get_value_mut(&mut self) -> &mut OptValue {
        &mut self.value
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

    fn parse_value(&self, string: &str) -> Result<OptValue> {
        let mut real_value = OptValue::from(vec![]);
        real_value.app_str(String::from(string));
        Ok(real_value)
    }

    fn has_value(&self) -> bool {
        self.get_value().is_vec()
    }

    fn reset_value(&mut self) {
        self.value = self.get_default_value().clone();
    }
}

impl Help for ArrayOpt {
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
pub struct ArrayCreator;

impl Creator for ArrayCreator {
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
        if create_info.get_prefix().is_none() {
            return Err(Error::NeedValidPrefix(current_type()));
        }

        assert_eq!(create_info.get_type_name(), self.get_type_name());

        let opt: ArrayOpt = create_info.into();

        Ok(Box::new(opt))
    }
}
