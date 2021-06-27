
use crate::opt::{
    Opt,
    index::Index,
    value::Value, 
    callback::CallbackType, 
    help::HelpInfo
};
use crate::uid::Uid;
use crate::err::Result;

pub trait Creator {
    fn get_type_name(&self) -> &'static str;

    fn is_support_deactivate_style(&self) -> bool;

    fn create_with(&self, id: Uid, create_info: CreateInfo) -> Result<Box<dyn Opt>>;
}

#[derive(Debug, Clone, Default)]
pub struct CreateInfo {
    deactivate_style: bool,

    optional: bool,

    type_name: String,

    name: String,

    prefix: String,

    index: Index,

    value: Value,

    alias: Vec<(String, String)>,

    callback_type: CallbackType,

    help: HelpInfo,
}

impl CreateInfo {
    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.deactivate_style = deactivate_style;
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn set_type_name(&mut self, type_name: String) -> &mut Self {
        self.type_name = type_name;
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_prefix(&mut self, prefix: String) -> &mut Self {
        self.prefix = prefix;
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.index = index;
        self
    }

    pub fn set_default_value(&mut self, value: Value) -> &mut Self {
        self.value = value;
        self
    }

    pub fn set_callback_type(&mut self, callback_type: CallbackType) -> &mut Self {
        self.callback_type = callback_type;
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.help.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.help.set_help(help);
        self
    }

    pub fn set_help_info(&mut self, help_info: HelpInfo) -> &mut Self {
        self.help = help_info;
        self
    }

    pub fn add_alias(&mut self, prefix: String, name: String) -> &mut Self {
        self.alias.push((prefix, name));
        self
    }

    pub fn rem_alias(&mut self, prefix: &str, name: &str) -> &mut Self {
        for (index, alias) in self.alias.iter().enumerate() {
            if alias.0 == prefix && alias.1 == name {
                self.alias.remove(index);
                break;
            }
        }
        self
    }

    pub fn clr_alias(&mut self) -> &mut Self {
        self.alias.clear();
        self
    }

    pub fn get_deactivate_style(&self) -> bool {
        self.deactivate_style
    }

    pub fn get_optional(&self) -> bool {
        self.optional
    }

    pub fn get_type_name(&self) -> &String {
        & self.type_name
    }

    pub fn get_name(&self) -> &String {
        & self.name
    }

    pub fn get_prefix(&self) -> &String {
        & self.prefix
    }

    pub fn get_index(&self) -> &Index {
        &self.index
    }

    pub fn get_alias(&self) -> &Vec<(String, String)> {
        self.alias.as_ref()
    }

    pub fn get_default_value(&self) -> &Value {
        &self.value
    }

    pub fn get_callback_type(&self) -> &CallbackType {
        &self.callback_type
    }

    pub fn get_help_info(&self) -> &HelpInfo {
        &self.help
    }

    pub fn get_deactivate_style_mut(&mut self) -> &mut bool {
        &mut self.deactivate_style
    }

    pub fn get_optional_mut(&mut self) -> &mut bool {
        &mut self.optional
    }

    pub fn get_type_name_mut(&mut self) -> &mut String {
        &mut self.type_name
    }

    pub fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn get_prefix_mut(&mut self) -> &mut String {
        &mut self.prefix
    }

    pub fn get_index_mut(&mut self) -> &mut Index {
        &mut self.index
    }

    pub fn get_alias_mut(&mut self) -> &mut Vec<(String, String)> {
        self.alias.as_mut()
    }

    pub fn get_default_value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn get_callback_type_mut(&mut self) -> &mut CallbackType {
        &mut self.callback_type
    }

    pub fn get_help_info_mut(&mut self) -> &mut HelpInfo {
        &mut self.help
    }
}