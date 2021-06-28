
use crate::opt::{
    Opt,
    index::Index,
    value::Value, 
    callback::CallbackType, 
    help::HelpInfo
};
use crate::proc::Info;
use crate::uid::Uid;
use crate::err::{Error, Result};
use crate::opt::parser::parse_option_str;

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

    pub fn parse(pattern: &str, prefix: &Vec<String>) -> Result<Self> {
        let mut data_keeper = parse_option_str(pattern, prefix)?;
        let index = data_keeper.gen_index();
        
        if data_keeper.prefix.is_some() {
            if data_keeper.name.is_some() {
                if data_keeper.type_name.is_some() {
                    return Ok(Self {
                        prefix: data_keeper.prefix.unwrap().clone(),
                        name: data_keeper.name.take().unwrap(),
                        type_name: data_keeper.type_name.take().unwrap(),
                        index,
                        deactivate_style: data_keeper.deactivate,
                        optional: ! data_keeper.optional,
                        .. Self::default()
                    })
                }
            }
        }
        Err(Error::InvalidOptionCreateString(String::from(pattern)))
    }
}


#[derive(Debug, Clone, Default)]
pub struct FilterInfo {
    deactivate_style: Option<bool>,

    optional: Option<bool>,

    type_name: Option<String>,

    name: Option<String>,

    prefix: Option<String>,

    index: Option<Index>,
}

impl FilterInfo {
    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.deactivate_style = Some(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = Some(optional);
        self
    }

    pub fn set_type_name(&mut self, type_name: String) -> &mut Self {
        self.type_name = Some(type_name);
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn set_prefix(&mut self, prefix: String) -> &mut Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.index = Some(index);
        self
    }

    pub fn get_deactivate_style(&self) -> bool {
        self.deactivate_style.unwrap()
    }

    pub fn get_optional(&self) -> bool {
        self.optional.unwrap()
    }

    pub fn get_type_name(&self) -> &String {
        self.type_name.as_ref().unwrap()
    }

    pub fn get_name(&self) -> &String {
        self.name.as_ref().unwrap()
    }

    pub fn get_prefix(&self) -> &String {
        self.prefix.as_ref().unwrap()
    }

    pub fn get_index(&self) -> &Index {
        self.index.as_ref().unwrap()
    }

    pub fn has_deactivate_style(&self) -> bool {
        self.deactivate_style.is_some()
    }

    pub fn has_optional(&self) -> bool {
        self.optional.is_some()
    }

    pub fn has_type_name(&self) -> bool {
        self.type_name.is_some()
    }

    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }

    pub fn has_prefix(&self) -> bool {
        self.prefix.is_some()
    }

    pub fn has_index(&self) -> bool {
        self.index.is_some()
    }

    pub fn match_opt(&self, opt: &dyn Opt) -> bool {
        let mut ret = true;

        if ret && self.has_type_name() {
            ret = ret && (self.get_type_name() == opt.get_type_name());
        }
        if ret && self.has_prefix() {
            let mut matched = opt.match_prefix(self.get_prefix());

            if ! matched {
                if let Some(alias) = opt.get_alias().as_ref() {
                    for item in alias.iter() {
                        if item.0 == self.get_prefix() {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_name() {
            let mut matched = opt.match_name(self.get_name());

            if ! matched {
                if let Some(alias) = opt.get_alias().as_ref() {
                    for item in alias.iter() {
                        if item.1 == self.get_name() {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_index() {
            ret = ret && (self.get_index() == opt.get_index());
        }
        ret
    }
}

#[derive(Debug)]
pub struct OptionInfo {
    uid: Uid,
}

impl From<Uid> for OptionInfo {
    fn from(v: Uid) -> Self {
        Self { uid: v }
    }
}

impl Info for OptionInfo {
    fn uid(&self) -> Uid {
        self.uid
    }
}