use std::mem::take;

use crate::err::SpecialError;
use crate::opt::*;
use crate::set::CreateInfo;
use crate::set::Creator;
use crate::uid::Uid;
use crate::Ustr;

pub mod path {
    use crate::err::{ConstructError, ParserError};

    use super::*;

    pub fn current_type() -> Ustr {
        Ustr::from("p")
    }

    pub trait Path: Opt {}

    #[derive(Debug)]
    pub struct PathOpt {
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

    impl From<CreateInfo> for PathOpt {
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

    impl Path for PathOpt {}

    impl Opt for PathOpt {}

    impl Type for PathOpt {
        fn get_type_name(&self) -> Ustr {
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

        fn check(&self) -> Result<()> {
            if !(self.get_optional() || self.has_value()) {
                Err(SpecialError::OptionForceRequired(self.get_hint().to_owned()).into())
            } else {
                Ok(())
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl Identifier for PathOpt {
        fn get_uid(&self) -> Uid {
            self.uid
        }

        fn set_uid(&mut self, uid: Uid) {
            self.uid = uid;
        }
    }

    impl Callback for PathOpt {
        fn is_need_invoke(&self) -> bool {
            self.need_invoke
        }

        fn set_invoke(&mut self, invoke: bool) {
            self.need_invoke = invoke;
        }

        fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool {
            if self.is_need_invoke() {
                match callback_type {
                    CallbackType::Opt | CallbackType::OptMut => true,
                    _ => false,
                }
            } else {
                false
            }
        }

        fn set_callback_ret(&mut self, ret: Option<OptValue>) -> Result<()> {
            if let Some(ret) = ret {
                if !ret.is_any() {
                    return Err(ParserError::InvalidReturnValueOfCallback(format!(
                        "excepted OptValue::vec, found {:?}",
                        ret
                    ))
                    .into());
                }
                self.set_value(ret);
            }
            Ok(())
        }
    }

    impl Name for PathOpt {
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

    impl Optional for PathOpt {
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

    impl Alias for PathOpt {
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
            self.alias
                .iter()
                .any(|&v| v.0 == prefix && v.1 == name)
        }
    }

    impl Index for PathOpt {
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

    impl Value for PathOpt {
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

        fn parse_value(&self, string: Ustr) -> Result<OptValue> {
            use std::path::PathBuf;
            Ok(OptValue::from_any(Box::new(PathBuf::from(string.as_ref()))))
        }

        fn has_value(&self) -> bool {
            self.get_value().is_int()
        }

        fn reset_value(&mut self) {
            // need fix in real code!
            self.value.reset();
        }
    }

    impl Help for PathOpt {
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

    #[derive(Debug, Default, Clone)]
    pub struct PathCreator;

    impl Creator for PathCreator {
        fn get_type_name(&self) -> Ustr {
            current_type()
        }

        fn is_support_deactivate_style(&self) -> bool {
            false
        }

        fn create_with(&self, create_info: CreateInfo) -> Result<Box<dyn Opt>> {
            if create_info.get_support_deactivate_style() {
                if !self.is_support_deactivate_style() {
                    return Err(ConstructError::NotSupportDeactivateStyle(
                        create_info.get_name().to_owned(),
                    )
                    .into());
                }
            }
            if create_info.get_prefix().is_none() {
                return Err(ConstructError::MissingOptionPrefix(current_type().to_owned()).into());
            }

            assert_eq!(create_info.get_type_name(), self.get_type_name());

            let opt: PathOpt = create_info.into();

            trace!(?opt, "create a Path");
            Ok(Box::new(opt))
        }
    }
}
