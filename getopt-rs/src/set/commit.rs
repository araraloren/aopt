
use crate::opt::index::Index;
use crate::opt::value::Value;
use crate::opt::callback::CallbackType;
use crate::opt::help::HelpInfo;
use super::info::CreateInfo;
use super::{Uid, Set, Result};

#[derive(Debug)]
pub struct Commit<'a> {
    set: &'a mut dyn Set,

    info: CreateInfo,
}

impl<'a> Commit<'a> {
    pub fn new(set: &'a mut dyn Set, info: CreateInfo) -> Self {
        Self {
            set, info,
        }
    }

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }    

    pub fn set_type_name(&mut self, type_name: String) -> &mut Self {
        self.info.set_type_name(type_name);
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.info.set_name(name);
        self
    }

    pub fn set_prefix(&mut self, prefix: String) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.info.set_index(index);
        self
    }

    pub fn set_default_value(&mut self, value: Value) -> &mut Self {
        self.info.set_default_value(value);
        self
    }

    pub fn set_callback_type(&mut self, callback_type: CallbackType) -> &mut Self {
        self.info.set_callback_type(callback_type);
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.info.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.info.set_help(help);
        self
    }

    pub fn set_help_info(&mut self, help_info: HelpInfo) -> &mut Self {
        self.info.set_help_info(help_info);
        self
    }

    pub fn add_alias(&mut self, prefix: String, name: String) -> &mut Self {
        self.info.add_alias(prefix, name);
        self
    }

    pub fn rem_alias(&mut self, prefix: &str, name: &str) -> &mut Self {
        self.info.rem_alias(prefix, name);
        self
    }

    pub fn clr_alias(&mut self) -> &mut Self {
        self.info.clr_alias();
        self
    }

    pub fn commit(&mut self) -> Result<Uid> {
        self.set.add_opt_ci(std::mem::take(&mut self.info))
    }
}