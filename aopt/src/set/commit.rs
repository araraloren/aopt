use super::info::CreateInfo;
use super::{Result, Set, Uid};

use crate::opt::HelpInfo;
use crate::opt::OptIndex;
use crate::opt::OptValue;
use crate::{gstr, Ustr};

#[derive(Debug)]
pub struct Commit<'a> {
    set: &'a mut dyn Set,

    info: CreateInfo,
}

impl<'a> Commit<'a> {
    pub fn new(set: &'a mut dyn Set, info: CreateInfo) -> Self {
        Self { set, info }
    }

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_support_deactivate_style(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    pub fn set_type_name(&mut self, type_name: &str) -> &mut Self {
        self.info.set_type_name(gstr(type_name));
        self
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.info.set_name(gstr(name));
        self
    }

    pub fn set_prefix(&mut self, prefix: &str) -> &mut Self {
        self.info.set_prefix(gstr(prefix));
        self
    }

    pub fn set_index(&mut self, index: OptIndex) -> &mut Self {
        self.info.set_index(index);
        self
    }

    pub fn set_default_value(&mut self, value: OptValue) -> &mut Self {
        self.info.set_default_value(value);
        self
    }

    pub fn set_hint(&mut self, hint: &str) -> &mut Self {
        self.info.set_hint(gstr(hint));
        self
    }

    pub fn set_help(&mut self, help: &str) -> &mut Self {
        self.info.set_help(gstr(help));
        self
    }

    pub fn set_help_info(&mut self, help_info: HelpInfo) -> &mut Self {
        self.info.set_help_info(help_info);
        self
    }

    pub fn add_alias(&mut self, alias: &str) -> Result<&mut Self> {
        self.info.add_alias(gstr(alias))?;
        Ok(self)
    }

    pub fn rem_alias(&mut self, alias: Ustr) -> &mut Self {
        self.info.rem_alias(alias);
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
