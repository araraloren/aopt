use serde::ser::SerializeStruct;
use serde::Serialize;
use std::any::Any;

use super::Information;
use super::OptCallback;
use crate::astr;
use crate::err::Error;
use crate::opt::OptHelp;
use crate::opt::OptIndex;
use crate::opt::OptParser;
use crate::set::Prefixed;
use crate::Str;
use crate::Uid;

pub trait Config {
    fn new<Parser>(parser: &Parser, pattern: Str) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser + Prefixed,
        Parser::Output: Information;
}

pub trait ConfigValue {
    fn get_uid(&self) -> Uid;

    fn get_name(&self) -> Option<Str>;

    fn get_prefix(&self) -> Option<Str>;

    fn get_type_name(&self) -> Option<Str>;

    fn get_index(&self) -> Option<&OptIndex>;

    fn get_alias(&self) -> Option<&Vec<Str>>;

    fn get_optional(&self) -> Option<bool>;

    fn get_hint(&self) -> Str;

    fn get_help(&self) -> Str;

    fn get_deactivate_style(&self) -> Option<bool>;

    fn get_support_prefix(&self) -> &Vec<Str>;

    fn get_callback<T>(&self) -> Option<&OptCallback<T>>
    where
        T: 'static;

    fn set_uid(&mut self, uid: Uid) -> &mut Self;

    fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self;

    fn set_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self;

    fn set_type_name<S: Into<Str>>(&mut self, type_name: S) -> &mut Self;

    fn set_index(&mut self, index: OptIndex) -> &mut Self;

    fn add_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn clr_alias(&mut self) -> &mut Self;

    fn rem_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn set_optional(&mut self, optional: bool) -> &mut Self;

    fn set_hint<S: Into<Str>>(&mut self, hint: S) -> &mut Self;

    fn set_help<S: Into<Str>>(&mut self, help: S) -> &mut Self;

    fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self;

    fn set_support_prefix<S: Into<Str>>(&mut self, prefix: Vec<S>) -> &mut Self;

    fn set_callback<T>(&mut self, callback: OptCallback<T>) -> &mut Self;

    fn has_name(&self) -> bool;

    fn has_prefix(&self) -> bool;

    fn has_type_name(&self) -> bool;

    fn has_index(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_optional(&self) -> bool;

    fn has_deactivate_style(&self) -> bool;

    fn has_callback(&self) -> bool;

    fn gen_uid(&self) -> Uid;

    fn gen_name(&self) -> Result<Str, Error>;

    fn gen_prefix(&self) -> Result<Str, Error>;

    fn gen_type_name(&self) -> Result<Str, Error>;

    fn gen_index(&self) -> Result<OptIndex, Error>;

    fn gen_alias(&self) -> Result<Vec<(Str, Str)>, Error>;

    fn gen_optional(&self) -> Result<bool, Error>;

    fn gen_opt_help(&self, deactivate_style: bool) -> Result<OptHelp, Error>;

    fn gen_deactivate_style(&self) -> Result<bool, Error>;

    fn take_uid(&mut self) -> Uid;

    fn take_name(&mut self) -> Option<Str>;

    fn take_prefix(&mut self) -> Option<Str>;

    fn take_type_name(&mut self) -> Option<Str>;

    fn take_index(&mut self) -> Option<OptIndex>;

    fn take_alias(&mut self) -> Option<Vec<Str>>;

    fn take_optional(&mut self) -> Option<bool>;

    fn take_opt_help(&mut self) -> OptHelp;

    fn take_deactivate_style(&mut self) -> Option<bool>;

    fn take_callback<T>(&mut self) -> Option<OptCallback<T>>
    where
        T: 'static;
}

/// Contain the information used for create option instance.
#[derive(Debug, Default)]
pub struct OptConfig {
    type_name: Option<Str>,

    uid: Uid,

    name: Option<Str>,

    prefix: Option<Str>,

    optional: Option<bool>,

    index: Option<OptIndex>,

    alias: Vec<Str>,

    help: OptHelp,

    support_prefix: Vec<Str>,

    deactivate_style: Option<bool>,

    callback: Option<Box<dyn Any>>,
}

/// Notice: callback will not serialized.
impl Serialize for OptConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("OptConfig", 11)?;
        let callback_none: Option<bool> = None;

        s.serialize_field("type_name", &self.type_name)?;
        s.serialize_field("uid", &self.uid)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("prefix", &self.prefix)?;
        s.serialize_field("optional", &self.optional)?;
        s.serialize_field("index", &self.index)?;
        s.serialize_field("alias", &self.alias)?;
        s.serialize_field("help", &self.help)?;
        s.serialize_field("support_prefix", &self.support_prefix)?;
        s.serialize_field("deactivate_style", &self.deactivate_style)?;
        s.serialize_field("callback", &callback_none)?;
        s.end()
    }
}

impl OptConfig {
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    pub fn with_deactivate_style(mut self, deactivate_style: bool) -> Self {
        self.deactivate_style = Some(deactivate_style);
        self
    }

    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = Some(optional);
        self
    }

    pub fn with_type_name(mut self, type_name: Str) -> Self {
        self.type_name = Some(type_name);
        self
    }

    pub fn with_name(mut self, name: Str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_prefix(mut self, prefix: Option<Str>) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_index(mut self, index: OptIndex) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_alias(mut self, alias: Vec<Str>) -> Self {
        self.alias = alias;
        self
    }

    pub fn with_help(mut self, help: OptHelp) -> Self {
        self.help = help;
        self
    }

    pub fn with_support_prefix(mut self, support_prefix: Vec<Str>) -> Self {
        self.support_prefix = support_prefix;
        self
    }

    pub fn with_callback<T>(mut self, mut callback: OptCallback<T>) -> Self
    where
        T: 'static,
    {
        self.callback = Some(callback.into_any());
        self
    }

    pub fn raise_missing_error(&self, field: &str) -> Result<Error, Error> {
        Ok(Error::con_missing_field(
            &astr(field),
            self.name
                .as_ref()
                .ok_or_else(|| Error::raise_error("Option type name can't be empty"))?,
            self.type_name
                .as_ref()
                .ok_or_else(|| Error::raise_error("Option name can't be empty"))?,
        ))
    }
}

impl Config for OptConfig {
    fn new<Parser>(parser: &Parser, pattern: Str) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser + Prefixed,
        Parser::Output: Information,
    {
        let mut output = parser.parse(pattern).map_err(|e| e.into())?;
        let mut ret = Self::default();

        if let Some(v) = output.take_name() {
            ret.set_name(v);
        }
        if let Some(v) = output.take_prefix() {
            ret.set_prefix(v);
        }
        if let Some(v) = output.take_type_name() {
            ret.set_type_name(v);
        }
        if let Some(v) = output.take_index() {
            ret.set_index(v);
        }
        if let Some(v) = output.take_optional() {
            ret.set_optional(!v);
        }
        if let Some(v) = output.take_deactivate_style() {
            ret.set_deactivate_style(v);
        }
        // set the prefix, it will use later
        ret.set_support_prefix(parser.get_prefix().to_vec());

        Ok(ret)
    }
}

impl ConfigValue for OptConfig {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn get_name(&self) -> Option<Str> {
        self.name.clone()
    }

    fn get_prefix(&self) -> Option<Str> {
        self.prefix.clone()
    }

    fn get_type_name(&self) -> Option<Str> {
        self.type_name.clone()
    }

    fn get_index(&self) -> Option<&OptIndex> {
        self.index.as_ref()
    }

    fn get_alias(&self) -> Option<&Vec<Str>> {
        Some(self.alias.as_ref())
    }

    fn get_optional(&self) -> Option<bool> {
        self.optional
    }

    fn get_hint(&self) -> Str {
        self.help.get_hint()
    }

    fn get_help(&self) -> Str {
        self.help.get_help()
    }

    fn get_deactivate_style(&self) -> Option<bool> {
        self.deactivate_style
    }

    fn get_support_prefix(&self) -> &Vec<Str> {
        &self.support_prefix
    }

    fn get_callback<T>(&self) -> Option<&OptCallback<T>>
    where
        T: 'static,
    {
        self.callback
            .as_ref()
            .and_then(|cb| cb.downcast_ref::<OptCallback<T>>())
    }

    fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    fn set_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.prefix = Some(prefix.into());
        self
    }

    fn set_type_name<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.type_name = Some(type_name.into());
        self
    }

    fn set_index(&mut self, index: OptIndex) -> &mut Self {
        self.index = Some(index);
        self
    }

    fn add_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self {
        self.alias.push(alias.into());
        self
    }

    fn clr_alias(&mut self) -> &mut Self {
        self.alias.clear();
        self
    }

    fn rem_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self {
        let alias = alias.into();

        for (index, value) in self.alias.iter().enumerate() {
            if value == &alias {
                self.alias.remove(index);
                break;
            }
        }
        self
    }

    fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = Some(optional);
        self
    }

    fn set_hint<S: Into<Str>>(&mut self, hint: S) -> &mut Self {
        self.help.set_hint(hint);
        self
    }

    fn set_help<S: Into<Str>>(&mut self, help: S) -> &mut Self {
        self.help.set_help(help);
        self
    }

    fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.deactivate_style = Some(deactivate_style);
        self
    }

    fn set_support_prefix<S: Into<Str>>(&mut self, prefix: Vec<S>) -> &mut Self {
        self.support_prefix = prefix.into_iter().map(|v| v.into()).collect();
        self
    }

    fn set_callback<T>(&mut self, mut callback: OptCallback<T>) -> &mut Self
    where
        T: 'static,
    {
        self.callback = Some(callback.into_any());
        self
    }

    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_prefix(&self) -> bool {
        self.prefix.is_some()
    }

    fn has_type_name(&self) -> bool {
        self.type_name.is_some()
    }

    fn has_index(&self) -> bool {
        self.index.is_some()
    }

    fn has_alias(&self) -> bool {
        !self.alias.is_empty()
    }

    fn has_optional(&self) -> bool {
        self.optional.is_some()
    }

    fn has_deactivate_style(&self) -> bool {
        self.deactivate_style.is_some()
    }

    fn has_callback(&self) -> bool {
        self.callback.is_some()
    }

    fn gen_uid(&self) -> Uid {
        self.uid
    }

    fn gen_name(&self) -> Result<Str, Error> {
        if let Some(name) = &self.name {
            return Ok(name.clone());
        }
        Err(self.raise_missing_error("name")?)
    }

    fn gen_prefix(&self) -> Result<Str, Error> {
        if let Some(prefix) = &self.prefix {
            return Ok(prefix.clone());
        }
        Err(self.raise_missing_error("prefix")?)
    }

    fn gen_type_name(&self) -> Result<Str, Error> {
        if let Some(type_name) = &self.type_name {
            return Ok(type_name.clone());
        }
        Err(self.raise_missing_error("type name")?)
    }

    fn gen_index(&self) -> Result<OptIndex, Error> {
        if let Some(index) = self.index.as_ref() {
            return Ok(index.clone());
        }
        Err(Error::con_missing_index(
            self.gen_name()?,
            self.gen_type_name()?,
        ))
    }

    fn gen_alias(&self) -> Result<Vec<(Str, Str)>, Error> {
        let mut ret = vec![];

        for alias in self.alias.iter() {
            let mut found_prefix = false;

            for prefix in self.support_prefix.iter() {
                if alias.starts_with(prefix.as_ref()) {
                    if let Some(name) = alias.get(prefix.len()..) {
                        ret.push((prefix.clone(), name.into()));
                        found_prefix = true;
                        break;
                    }
                }
            }
            if !found_prefix {
                return Err(Error::con_invalid_option_alias(alias));
            }
        }
        Ok(ret)
    }

    fn gen_optional(&self) -> Result<bool, Error> {
        if let Some(optional) = self.optional {
            return Ok(optional);
        }
        Err(self.raise_missing_error("optional")?)
    }

    fn gen_opt_help(&self, deactivate_style: bool) -> Result<OptHelp, Error> {
        let mut ret = self.help.clone();

        if ret.get_hint().is_empty() {
            let mut names = vec![String::default()];

            // add prefix
            if let Some(prefix) = self.get_prefix() {
                names[0] += prefix.as_str();
            }
            // add deactivate style
            if deactivate_style {
                names[0] += "/";
            }
            // add name
            names[0] += self.gen_name()?.as_ref();

            // add alias
            if let Some(alias_vec) = self.get_alias() {
                for alias in alias_vec {
                    if deactivate_style {
                        for prefix in self.get_support_prefix() {
                            if alias.starts_with(prefix.as_str()) {
                                if let Some(name) = alias.get(prefix.len()..alias.len()) {
                                    names.push(format!("{}/{}", prefix, name));
                                    break;
                                }
                            }
                        }
                    } else {
                        names.push(format!("{}", alias));
                    }
                }
            }
            // sort name by len
            names.sort_by_key(|v| v.len());

            if let Some(index) = &self.index {
                let index_string = index.to_help();

                // add index string
                if index_string.is_empty() {
                    ret.set_hint(names.join(","));
                } else {
                    ret.set_hint(format!("{}@{}", names.join(","), index_string));
                }
            } else {
                ret.set_hint(names.join(","));
            }
        }
        Ok(ret)
    }

    fn gen_deactivate_style(&self) -> Result<bool, Error> {
        if let Some(deactivate_style) = self.deactivate_style {
            return Ok(deactivate_style);
        }
        Err(self.raise_missing_error("deactivate_style")?)
    }

    fn take_uid(&mut self) -> Uid {
        self.uid
    }

    fn take_name(&mut self) -> Option<Str> {
        self.name.take()
    }

    fn take_prefix(&mut self) -> Option<Str> {
        self.prefix.take()
    }

    fn take_type_name(&mut self) -> Option<Str> {
        self.type_name.take()
    }

    fn take_index(&mut self) -> Option<OptIndex> {
        self.index.take()
    }

    fn take_alias(&mut self) -> Option<Vec<Str>> {
        Some(std::mem::take(&mut self.alias))
    }

    fn take_optional(&mut self) -> Option<bool> {
        self.optional.take()
    }

    fn take_opt_help(&mut self) -> OptHelp {
        std::mem::take(&mut self.help)
    }

    fn take_deactivate_style(&mut self) -> Option<bool> {
        self.deactivate_style.take()
    }

    fn take_callback<T>(&mut self) -> Option<OptCallback<T>>
    where
        T: 'static,
    {
        if let Some(callback) = self.callback.take() {
            if let Ok(callback) = callback.downcast::<OptCallback<T>>() {
                return Some(*callback);
            }
        }
        None
    }
}
