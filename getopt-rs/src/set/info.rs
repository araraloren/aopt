use crate::err::{ConstructError, Result};
use crate::opt::parser::parse_option_str;
use crate::opt::parser::DataKeeper;
use crate::opt::{HelpInfo, Opt, OptIndex, OptValue};
use crate::proc::Info;
use crate::uid::Uid;
use crate::Ustr;

#[derive(Debug, Clone, Default)]
pub struct CreateInfo {
    uid: Uid,

    support_deactivate_style: bool,

    optional: bool,

    type_name: Ustr,

    name: Ustr,

    prefix: Option<Ustr>,

    index: OptIndex,

    value: OptValue,

    has_value: bool,

    alias: Vec<Ustr>,

    help: HelpInfo,

    support_prefix: Vec<Ustr>,
}

impl CreateInfo {
    pub fn set_support_prefix(&mut self, prefix: Vec<Ustr>) {
        self.support_prefix = prefix;
    }

    pub fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    pub fn set_support_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.support_deactivate_style = deactivate_style;
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn set_type_name(&mut self, type_name: Ustr) -> &mut Self {
        self.type_name = type_name;
        self
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_prefix(&mut self, prefix: Ustr) -> &mut Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn set_index(&mut self, index: OptIndex) -> &mut Self {
        self.index = index;
        self
    }

    pub fn set_default_value(&mut self, value: OptValue) -> &mut Self {
        self.value = value;
        self
    }

    pub fn set_hint(&mut self, hint: Ustr) -> &mut Self {
        self.help.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
        self.help.set_help(help);
        self
    }

    pub fn set_help_info(&mut self, help_info: HelpInfo) -> &mut Self {
        self.help = help_info;
        self
    }

    pub fn add_alias(&mut self, alias: Ustr) -> Result<&mut Self> {
        let has_prefix = self
            .support_prefix
            .iter()
            .any(|v| alias.starts_with(v.as_ref()) && alias.len() != v.len());

        if has_prefix {
            self.alias.push(alias);
            Ok(self)
        } else {
            Err(ConstructError::InvalidOptionAlias(alias.to_string()).into())
        }
    }

    pub fn rem_alias(&mut self, alias: Ustr) -> &mut Self {
        for (index, value) in self.alias.iter().enumerate() {
            if value == &alias {
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

    pub fn gen_option_alias(&self) -> Vec<(Ustr, Ustr)> {
        let mut ret = vec![];

        for alias in self.alias.iter() {
            for prefix in self.support_prefix.iter() {
                if alias.starts_with(prefix.as_ref()) {
                    if let Some(name) = alias.get(prefix.len()..) {
                        ret.push((prefix.clone(), name.into()));
                    }
                }
            }
        }
        ret
    }

    pub fn get_uid(&self) -> Uid {
        self.uid
    }

    pub fn get_support_deactivate_style(&self) -> bool {
        self.support_deactivate_style
    }

    pub fn get_optional(&self) -> bool {
        self.optional
    }

    pub fn get_type_name(&self) -> Ustr {
        self.type_name
    }

    pub fn get_name(&self) -> Ustr {
        self.name
    }

    pub fn get_prefix(&self) -> Option<Ustr> {
        self.prefix
    }

    pub fn get_index(&self) -> &OptIndex {
        &self.index
    }

    pub fn get_alias(&self) -> &Vec<Ustr> {
        self.alias.as_ref()
    }

    pub fn get_default_value(&self) -> &OptValue {
        &self.value
    }

    pub fn get_help_info(&self) -> &HelpInfo {
        &self.help
    }

    pub fn get_deactivate_style_mut(&mut self) -> &mut bool {
        &mut self.support_deactivate_style
    }

    pub fn get_optional_mut(&mut self) -> &mut bool {
        &mut self.optional
    }

    pub fn get_type_name_mut(&mut self) -> &mut Ustr {
        &mut self.type_name
    }

    pub fn get_name_mut(&mut self) -> &mut Ustr {
        &mut self.name
    }

    pub fn get_prefix_mut(&mut self) -> &mut Option<Ustr> {
        &mut self.prefix
    }

    pub fn get_index_mut(&mut self) -> &mut OptIndex {
        &mut self.index
    }

    pub fn get_alias_mut(&mut self) -> &mut Vec<Ustr> {
        self.alias.as_mut()
    }

    pub fn get_default_value_mut(&mut self) -> &mut OptValue {
        &mut self.value
    }

    pub fn get_help_info_mut(&mut self) -> &mut HelpInfo {
        &mut self.help
    }

    pub fn parse(pattern: Ustr, prefix: &[Ustr]) -> Result<Self> {
        let data_keeper = parse_option_str(pattern, prefix)?;

        if data_keeper.name.is_none() {
            return Err(ConstructError::MissingOptionName(pattern.to_owned()).into());
        }
        if data_keeper.type_name.is_none() {
            return Err(ConstructError::MissingOptionType(pattern.to_owned()).into());
        }
        let mut ret: Self = data_keeper.into();
        ret.set_support_prefix(prefix.iter().map(|v| v.clone()).collect::<Vec<Ustr>>());
        return Ok(ret);
    }
}

impl From<DataKeeper> for CreateInfo {
    fn from(data_keeper: DataKeeper) -> Self {
        let mut data_keeper = data_keeper;
        let index = data_keeper.gen_index();

        Self {
            prefix: data_keeper.prefix.map(|v| v.clone()),
            name: data_keeper.name.take().unwrap(),
            type_name: data_keeper.type_name.take().unwrap(),
            index,
            support_deactivate_style: data_keeper.deactivate.unwrap_or(false),
            optional: !data_keeper.optional.unwrap_or(false),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FilterInfo {
    deactivate_style: Option<bool>,

    optional: Option<bool>,

    type_name: Option<Ustr>,

    name: Option<Ustr>,

    prefix: Option<Ustr>,

    index: Option<OptIndex>,
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

    pub fn set_type_name(&mut self, type_name: Ustr) -> &mut Self {
        self.type_name = Some(type_name);
        self
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn set_prefix(&mut self, prefix: Ustr) -> &mut Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn set_index(&mut self, index: OptIndex) -> &mut Self {
        self.index = Some(index);
        self
    }

    pub fn get_deactivate_style(&self) -> bool {
        self.deactivate_style.unwrap()
    }

    pub fn get_optional(&self) -> bool {
        self.optional.unwrap()
    }

    pub fn get_type_name(&self) -> Ustr {
        self.type_name.clone().unwrap()
    }

    pub fn get_name(&self) -> Ustr {
        self.name.clone().unwrap()
    }

    pub fn get_prefix(&self) -> Ustr {
        self.prefix.clone().unwrap()
    }

    pub fn get_index(&self) -> &OptIndex {
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

    pub fn parse(pattern: Ustr, prefix: &[Ustr]) -> Result<Self> {
        Ok(parse_option_str(pattern, prefix)?.into())
    }

    pub fn match_opt(&self, opt: &dyn Opt) -> bool {
        let mut ret = true;

        if ret && self.has_deactivate_style() {
            ret = ret && (self.get_deactivate_style() == opt.is_deactivate_style());
        }
        if ret && self.has_optional() {
            ret = ret && (self.get_optional() == opt.get_optional());
        }
        if ret && self.has_type_name() {
            ret = ret && (self.get_type_name() == opt.get_type_name());
        }
        if ret && self.has_prefix() {
            // don't call match prefix
            let mut matched = opt.get_prefix() == self.get_prefix();

            if !matched {
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
            // don't call match name
            let mut matched = opt.get_name() == self.get_name();

            if !matched {
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
            if let Some(index) = opt.get_index() {
                ret = ret && (self.get_index() == index);
            }
        }
        ret
    }
}

impl From<DataKeeper> for FilterInfo {
    fn from(data_keeper: DataKeeper) -> Self {
        let mut data_keeper = data_keeper;
        let has_index = data_keeper.has_index();
        let index = data_keeper.gen_index();

        Self {
            prefix: data_keeper.prefix.map(|v| v.clone()),
            name: data_keeper.name,
            type_name: data_keeper.type_name,
            index: if has_index { Some(index) } else { None },
            deactivate_style: data_keeper.deactivate,
            optional: data_keeper.optional,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OptionInfo {
    uid: Uid,
}

impl From<Uid> for OptionInfo {
    fn from(v: Uid) -> Self {
        Self { uid: v }
    }
}

impl<'a> From<&'a dyn Opt> for OptionInfo {
    fn from(opt: &'a dyn Opt) -> Self {
        Self { uid: opt.get_uid() }
    }
}

impl Info for OptionInfo {
    fn info_uid(&self) -> Uid {
        self.uid
    }
}
