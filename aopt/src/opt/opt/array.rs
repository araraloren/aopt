use std::convert::{TryFrom, TryInto};
use std::mem::take;
use ustr::Ustr;

use crate::err::Error;
use crate::gstr;
use crate::opt::*;
use crate::set::CreateInfo;
use crate::set::Creator;
use crate::uid::Uid;

pub trait Array: Opt {}

#[derive(Debug)]
pub struct ArrayOpt {
    uid: Uid,

    name: Ustr,

    prefix: Ustr,

    optional: bool,

    value: OptValue,

    default_value: OptValue,

    alias: Vec<(Ustr, Ustr)>,

    need_invoke: bool,

    help_info: HelpInfo,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        unsafe impl Send for ArrayOpt { }
        unsafe impl Sync for ArrayOpt { }
    }
}

impl TryFrom<CreateInfo> for ArrayOpt {
    type Error = Error;

    fn try_from(value: CreateInfo) -> Result<Self> {
        let mut ci = value;
        let help_info = HelpInfo::from(&mut ci);
        let prefix = ci
            .get_prefix()
            .ok_or_else(|| Error::opt_missing_prefix(ci.get_name(), ci.get_type_name()))?;

        Ok(Self {
            uid: ci.get_uid(),
            name: ci.get_name(),
            prefix,
            optional: ci.get_optional(),
            value: OptValue::default(),
            default_value: take(ci.get_default_value_mut()),
            alias: ci.gen_option_alias(),
            need_invoke: false,
            help_info,
        })
    }
}

impl Array for ArrayOpt {}

impl Opt for ArrayOpt {}

impl Type for ArrayOpt {
    fn get_type_name(&self) -> Ustr {
        ArrayCreator::type_name()
    }

    fn is_deactivate_style(&self) -> bool {
        false
    }

    fn match_style(&self, style: Style) -> bool {
        matches!(style, Style::Argument)
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
        matches!(callback_type, CallbackType::Opt | CallbackType::OptMut)
    }

    fn set_callback_ret(&mut self, ret: Option<OptValue>) -> Result<()> {
        if let Some(mut ret) = ret {
            if !ret.is_vec() {
                return Err(Error::opt_invalid_ret_value(format!(
                    "excepted OptValue::vec, found {:?}",
                    ret
                )));
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
    fn get_name(&self) -> Ustr {
        self.name
    }

    fn get_prefix(&self) -> Ustr {
        self.prefix
    }

    fn set_name(&mut self, string: Ustr) {
        self.name = string;
    }

    fn set_prefix(&mut self, string: Ustr) {
        self.prefix = string;
    }

    fn match_name(&self, name: Ustr) -> bool {
        self.get_name() == name
    }

    fn match_prefix(&self, prefix: Ustr) -> bool {
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
    fn get_alias(&self) -> Option<&Vec<(Ustr, Ustr)>> {
        Some(&self.alias)
    }

    fn add_alias(&mut self, prefix: Ustr, name: Ustr) {
        self.alias.push((prefix, name));
    }

    fn rem_alias(&mut self, prefix: Ustr, name: Ustr) {
        for (index, value) in self.alias.iter().enumerate() {
            if value.0 == prefix && value.1 == name {
                self.alias.remove(index);
                break;
            }
        }
    }

    fn match_alias(&self, prefix: Ustr, name: Ustr) -> bool {
        self.alias.iter().any(|&v| v.0 == prefix && v.1 == name)
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

    fn parse_value(&self, string: Ustr, _disable: bool, _index: u64) -> Result<OptValue> {
        Ok(OptValue::from(vec![string.to_string()]))
    }

    fn has_value(&self) -> bool {
        self.get_value().is_vec()
    }

    fn reset_value(&mut self) {
        self.value = self.get_default_value().clone();
    }
}

impl Help for ArrayOpt {
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

/// [`Creator`] implementation of option type [`ArrayOpt`].
#[derive(Debug, Default, Clone)]
pub struct ArrayCreator;

impl ArrayCreator {
    pub fn type_name() -> Ustr {
        gstr("a")
    }
}

impl Creator for ArrayCreator {
    fn get_type_name(&self) -> Ustr {
        ArrayCreator::type_name()
    }

    fn is_support_deactivate_style(&self) -> bool {
        false
    }

    fn create_with(&self, create_info: CreateInfo) -> Result<Box<dyn Opt>> {
        if create_info.get_support_deactivate_style() && !self.is_support_deactivate_style() {
            return Err(Error::opt_unsupport_deactivate_style(
                create_info.get_name(),
            ));
        }

        assert_eq!(create_info.get_type_name(), self.get_type_name());

        let opt: ArrayOpt = create_info.try_into()?;

        trace!(?opt, "create a Array");
        Ok(Box::new(opt))
    }
}
