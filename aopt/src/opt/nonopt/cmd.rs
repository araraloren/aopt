use std::convert::{TryFrom, TryInto};

use super::NonOpt;

use crate::err::{ConstructError, Error, SpecialError};
use crate::opt::*;
use crate::set::{CreateInfo, Creator};
use crate::uid::Uid;
use crate::Ustr;

pub fn current_type() -> Ustr {
    Ustr::from("c")
}
pub trait Cmd: NonOpt {}

#[derive(Debug)]
pub struct CmdOpt {
    uid: Uid,

    name: Ustr,

    value: OptValue,

    index: OptIndex,

    need_invoke: bool,

    help_info: HelpInfo,
}

impl TryFrom<CreateInfo> for CmdOpt {
    type Error = Error;

    fn try_from(value: CreateInfo) -> Result<Self> {
        let mut ci = value;
        let help_info = HelpInfo::from(&mut ci);

        Ok(Self {
            uid: ci.get_uid(),
            name: ci.get_name().clone(),
            value: OptValue::Null,
            index: OptIndex::forward(1),
            need_invoke: false,
            help_info,
        })
    }
}

impl Cmd for CmdOpt {}

impl Opt for CmdOpt {}

impl NonOpt for CmdOpt {}

impl Type for CmdOpt {
    fn get_type_name(&self) -> Ustr {
        current_type()
    }

    fn is_deactivate_style(&self) -> bool {
        false
    }

    fn match_style(&self, style: Style) -> bool {
        match style {
            Style::Cmd => true,
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

impl Identifier for CmdOpt {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }
}

impl Callback for CmdOpt {
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

impl Name for CmdOpt {
    fn get_name(&self) -> Ustr {
        self.name
    }

    fn get_prefix(&self) -> Ustr {
        Ustr::from("")
    }

    fn set_name(&mut self, string: Ustr) {
        self.name = string;
    }

    fn set_prefix(&mut self, _string: Ustr) {}

    fn match_name(&self, name: Ustr) -> bool {
        self.get_name() == name
    }

    fn match_prefix(&self, _prefix: Ustr) -> bool {
        false
    }
}

impl Optional for CmdOpt {
    fn get_optional(&self) -> bool {
        false
    }

    fn set_optional(&mut self, _optional: bool) {}

    fn match_optional(&self, optional: bool) -> bool {
        self.get_optional() == optional
    }
}

impl Alias for CmdOpt {
    fn get_alias(&self) -> Option<&Vec<(Ustr, Ustr)>> {
        None
    }

    fn add_alias(&mut self, _prefix: Ustr, _name: Ustr) {}

    fn rem_alias(&mut self, _prefix: Ustr, _name: Ustr) {}

    fn match_alias(&self, _prefix: Ustr, _name: Ustr) -> bool {
        false
    }
}

impl Index for CmdOpt {
    fn get_index(&self) -> Option<&OptIndex> {
        Some(&self.index)
    }

    fn set_index(&mut self, _index: OptIndex) {}

    fn match_index(&self, total: u64, current: u64) -> bool {
        if let Some(realindex) = self.get_index() {
            if let Some(realindex) = realindex.calc_index(total, current) {
                return realindex == current;
            }
        }
        false
    }
}

impl Value for CmdOpt {
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

    /// Can't change the default value of non-opt
    fn set_default_value(&mut self, _value: OptValue) {}

    fn parse_value(&self, _string: Ustr) -> Result<OptValue> {
        Ok(OptValue::from(true))
    }

    fn has_value(&self) -> bool {
        !self.get_value().is_null()
    }

    fn reset_value(&mut self) {
        self.value = self.get_default_value().clone();
    }
}

impl Help for CmdOpt {
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
pub struct CmdCreator;

impl Creator for CmdCreator {
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
        assert_eq!(create_info.get_type_name(), self.get_type_name());
        let opt: CmdOpt = create_info.try_into()?;

        trace!(?opt, "create a Cmd");
        Ok(Box::new(opt))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_type_cmd_work() {
        let creator = CmdCreator::default();

        assert_eq!(creator.get_type_name(), current_type());
        // cmd not support deactivate style
        assert_eq!(creator.is_support_deactivate_style(), false);

        let mut ci = CreateInfo::parse(Ustr::from("cmd=c"), &[]).unwrap();

        ci.set_uid(1);

        let mut cmd = creator.create_with(ci).unwrap();

        assert_eq!(cmd.get_type_name(), current_type());
        assert_eq!(cmd.is_deactivate_style(), false);
        assert_eq!(cmd.match_style(Style::Cmd), true);
        assert_eq!(cmd.check().is_err(), true);

        assert_eq!(cmd.get_uid(), 1);
        cmd.set_uid(42);
        assert_eq!(cmd.get_uid(), 42);

        assert_eq!(cmd.is_need_invoke(), false);
        cmd.set_invoke(true);
        assert_eq!(cmd.is_accept_callback_type(CallbackType::Main), true);
        assert_eq!(cmd.is_accept_callback_type(CallbackType::MainMut), true);
        assert_eq!(cmd.is_need_invoke(), true);

        // cmd not support alias
        cmd.add_alias("-".into(), "m".into());
        assert_eq!(cmd.get_alias(), None);
        assert_eq!(cmd.match_alias("-".into(), "m".into()), false);
        cmd.rem_alias("-".into(), "m".into());
        assert_eq!(cmd.get_alias(), None);

        assert_eq!(cmd.get_index(), Some(&OptIndex::forward(1)));
        assert_eq!(cmd.match_index(6, 1), true);
        assert_eq!(cmd.match_index(6, 2), false);
        cmd.set_index(OptIndex::forward(3));
        assert_eq!(cmd.match_index(6, 1), true);
        assert_eq!(cmd.match_index(6, 3), false);
        assert_eq!(cmd.get_index(), Some(&OptIndex::forward(1)));
        assert_eq!(cmd.match_index(6, 9), false);

        assert_eq!(cmd.get_name(), Ustr::from("cmd"));
        assert_eq!(cmd.get_prefix(), Ustr::from(""));
        assert_eq!(cmd.match_name("www".into()), false);
        assert_eq!(cmd.match_name("cmd".into()), true);
        assert_eq!(cmd.match_prefix("--".into()), false);
        assert_eq!(cmd.match_prefix("".into()), false);
        cmd.set_name(Ustr::from("cmd1"));
        cmd.set_prefix(Ustr::from("+"));
        assert_eq!(cmd.match_name("www".into()), false);
        assert_eq!(cmd.match_name("cmd1".into()), true);
        assert_eq!(cmd.get_name(), Ustr::from("cmd1"));
        assert_eq!(cmd.match_prefix("+".into()), false);
        assert_eq!(cmd.match_prefix("".into()), false);

        assert_eq!(cmd.get_optional(), false);
        assert_eq!(cmd.match_optional(true), false);
        cmd.set_optional(true);
        assert_eq!(cmd.get_optional(), false);
        assert_eq!(cmd.match_optional(false), true);
        assert_eq!(cmd.check().is_err(), true);

        assert_eq!(cmd.get_value().is_null(), true);
        assert_eq!(cmd.get_default_value().is_null(), true);
        assert_eq!(cmd.has_value(), false);
        let value = cmd.parse_value("".into());
        assert_eq!(value.is_ok(), true);
        let value = value.unwrap();
        assert_eq!(value.is_bool(), true);
        cmd.set_value(value);
        assert_eq!(cmd.get_value().as_bool(), OptValue::from(true).as_bool());
        cmd.set_default_value(OptValue::from(false));
        assert_eq!(cmd.get_default_value().is_null(), true);
        cmd.reset_value();
        assert_eq!(cmd.get_value().is_null(), true);

        assert_eq!(cmd.as_ref().as_any().is::<CmdOpt>(), true);
    }
}
