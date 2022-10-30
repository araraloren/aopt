use crate::astr;
use crate::err::Error;
use crate::opt::Information;
use crate::opt::OptHelp;
use crate::opt::OptIndex;
use crate::opt::OptParser;
use crate::opt::ValPolicy;
use crate::opt::ValType;
use crate::set::Pre;
use crate::Str;
use crate::Uid;

pub trait Config {
    fn new<Parser>(parser: &Parser, pattern: Str) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser + Pre,
        Parser::Output: Information;
}

pub trait ConfigValue {
    fn uid(&self) -> Uid;

    fn name(&self) -> Option<&Str>;

    /// Option's prefix string.
    fn prefix(&self) -> Option<&Str>;

    /// Option's type name.
    fn r#type(&self) -> Option<&Str>;

    /// Option's index configuration.
    fn idx(&self) -> Option<&OptIndex>;

    fn alias(&self) -> Option<&Vec<Str>>;

    fn optional(&self) -> Option<bool>;

    fn hint(&self) -> &Str;

    fn help(&self) -> &Str;

    /// If option support deactivatet style.
    fn deactivate(&self) -> Option<bool>;

    /// Prefix string using for parsing alias.
    fn sp_prefix(&self) -> &Vec<Str>;

    fn policy(&self) -> Option<&(ValPolicy, ValType)>;

    fn set_uid(&mut self, uid: Uid) -> &mut Self;

    fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self;

    fn set_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self;

    fn set_type<S: Into<Str>>(&mut self, type_name: S) -> &mut Self;

    fn set_idx(&mut self, index: OptIndex) -> &mut Self;

    fn add_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn clr_alias(&mut self) -> &mut Self;

    fn rem_alias<S: Into<Str>>(&mut self, alias: S) -> &mut Self;

    fn set_opt(&mut self, optional: bool) -> &mut Self;

    fn set_hint<S: Into<Str>>(&mut self, hint: S) -> &mut Self;

    fn set_help<S: Into<Str>>(&mut self, help: S) -> &mut Self;

    fn set_deactivate(&mut self, deactivate_style: bool) -> &mut Self;

    fn set_spprefix<S: Into<Str>>(&mut self, prefix: Vec<S>) -> &mut Self;

    fn set_policy(&mut self, policy: Option<(ValPolicy, ValType)>) -> &mut Self;

    fn has_name(&self) -> bool;

    fn has_prefix(&self) -> bool;

    fn has_type(&self) -> bool;

    fn has_idx(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_optional(&self) -> bool;

    fn has_deactivate(&self) -> bool;

    fn has_policy(&self) -> bool;

    fn gen_uid(&self) -> Uid;

    fn gen_name(&self) -> Result<Str, Error>;

    fn gen_prefix(&self) -> Result<Str, Error>;

    fn gen_type(&self) -> Result<Str, Error>;

    fn gen_idx(&self) -> Result<OptIndex, Error>;

    fn gen_alias(&self) -> Result<Vec<(Str, Str)>, Error>;

    fn gen_optional(&self) -> Result<bool, Error>;

    fn gen_opt_help(&self, deactivate_style: bool) -> Result<OptHelp, Error>;

    fn gen_deactivate(&self) -> Result<bool, Error>;

    fn gen_policy(&self) -> Option<(ValPolicy, ValType)>;

    fn take_uid(&mut self) -> Uid;

    fn take_name(&mut self) -> Option<Str>;

    fn take_prefix(&mut self) -> Option<Str>;

    fn take_type(&mut self) -> Option<Str>;

    fn take_idx(&mut self) -> Option<OptIndex>;

    fn take_alias(&mut self) -> Option<Vec<Str>>;

    fn take_optional(&mut self) -> Option<bool>;

    fn take_opt_help(&mut self) -> OptHelp;

    fn take_deactivate(&mut self) -> Option<bool>;

    fn take_policy(&mut self) -> Option<(ValPolicy, ValType)>;
}

/// Contain the information used for create option instance.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OptConfig {
    ty: Option<Str>,

    uid: Uid,

    name: Option<Str>,

    pre: Option<Str>,

    opt: Option<bool>,

    idx: Option<OptIndex>,

    alias: Vec<Str>,

    help: OptHelp,

    sp_pre: Vec<Str>,

    deact: Option<bool>,

    policy: Option<(ValPolicy, ValType)>,
}

impl OptConfig {
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    pub fn with_deact(mut self, deactivate_style: bool) -> Self {
        self.deact = Some(deactivate_style);
        self
    }

    pub fn with_optional(mut self, optional: bool) -> Self {
        self.opt = Some(optional);
        self
    }

    pub fn with_type<S: Into<Str>>(mut self, type_name: S) -> Self {
        self.ty = Some(type_name.into());
        self
    }

    pub fn with_name<S: Into<Str>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_prefix<S: Into<Str>>(mut self, prefix: Option<S>) -> Self {
        self.pre = prefix.map(|v| v.into());
        self
    }

    pub fn with_idx(mut self, index: OptIndex) -> Self {
        self.idx = Some(index);
        self
    }

    pub fn with_alias<S: Into<Str>>(mut self, alias: Vec<S>) -> Self {
        self.alias = alias.into_iter().map(|v| v.into()).collect();
        self
    }

    pub fn with_hint<S: Into<Str>>(mut self, hint: S) -> Self {
        self.help.set_hint(hint.into());
        self
    }

    pub fn with_help<S: Into<Str>>(mut self, help: S) -> Self {
        self.help.set_help(help.into());
        self
    }

    pub fn with_spprefix<S: Into<Str>>(mut self, support_prefix: Vec<S>) -> Self {
        self.sp_pre = support_prefix.into_iter().map(|v| v.into()).collect();
        self
    }

    pub fn with_policy(mut self, policy: Option<(ValPolicy, ValType)>) -> Self {
        self.policy = policy;
        self
    }

    pub fn raise_missing_error(&self, field: &str) -> Result<Error, Error> {
        Ok(Error::con_missing_field(
            &astr(field),
            self.name
                .as_ref()
                .ok_or_else(|| Error::raise_error("Option type name can't be empty"))?,
            self.ty
                .as_ref()
                .ok_or_else(|| Error::raise_error("Option name can't be empty"))?,
        ))
    }
}

impl Config for OptConfig {
    fn new<Parser>(parser: &Parser, pattern: Str) -> Result<Self, Error>
    where
        Self: Sized,
        Parser: OptParser + Pre,
        Parser::Output: Information,
    {
        let mut output = parser.parse(pattern).map_err(|e| e.into())?;
        let mut ret = Self::default();

        if let Some(v) = output.take_name() {
            ret.set_name(v);
        }
        if let Some(v) = output.take_pre() {
            ret.set_prefix(v);
        }
        if let Some(v) = output.take_ty() {
            ret.set_type(v);
        }
        if let Some(v) = output.take_idx() {
            ret.set_idx(v);
        }
        if let Some(v) = output.take_opt() {
            ret.set_opt(!v);
        }
        if let Some(v) = output.take_deact() {
            ret.set_deactivate(v);
        }
        // set the prefix, it will use later
        ret.set_spprefix(parser.prefix().to_vec());

        Ok(ret)
    }
}

impl ConfigValue for OptConfig {
    fn uid(&self) -> Uid {
        self.uid
    }

    fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    fn prefix(&self) -> Option<&Str> {
        self.pre.as_ref()
    }

    fn r#type(&self) -> Option<&Str> {
        self.ty.as_ref()
    }

    fn idx(&self) -> Option<&OptIndex> {
        self.idx.as_ref()
    }

    fn alias(&self) -> Option<&Vec<Str>> {
        Some(self.alias.as_ref())
    }

    fn optional(&self) -> Option<bool> {
        self.opt
    }

    fn hint(&self) -> &Str {
        self.help.hint()
    }

    fn help(&self) -> &Str {
        self.help.help()
    }

    fn deactivate(&self) -> Option<bool> {
        self.deact
    }

    fn sp_prefix(&self) -> &Vec<Str> {
        &self.sp_pre
    }

    fn policy(&self) -> Option<&(ValPolicy, ValType)> {
        self.policy.as_ref()
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
        self.pre = Some(prefix.into());
        self
    }

    fn set_type<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.ty = Some(type_name.into());
        self
    }

    fn set_idx(&mut self, index: OptIndex) -> &mut Self {
        self.idx = Some(index);
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

    fn set_opt(&mut self, optional: bool) -> &mut Self {
        self.opt = Some(optional);
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

    fn set_deactivate(&mut self, deactivate_style: bool) -> &mut Self {
        self.deact = Some(deactivate_style);
        self
    }

    fn set_spprefix<S: Into<Str>>(&mut self, prefix: Vec<S>) -> &mut Self {
        self.sp_pre = prefix.into_iter().map(|v| v.into()).collect();
        self
    }

    fn set_policy(&mut self, policy: Option<(ValPolicy, ValType)>) -> &mut Self {
        self.policy = policy;
        self
    }

    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_prefix(&self) -> bool {
        self.pre.is_some()
    }

    fn has_type(&self) -> bool {
        self.ty.is_some()
    }

    fn has_idx(&self) -> bool {
        self.idx.is_some()
    }

    fn has_alias(&self) -> bool {
        !self.alias.is_empty()
    }

    fn has_optional(&self) -> bool {
        self.opt.is_some()
    }

    fn has_deactivate(&self) -> bool {
        self.deact.is_some()
    }

    fn has_policy(&self) -> bool {
        self.policy.is_some()
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
        if let Some(prefix) = &self.pre {
            return Ok(prefix.clone());
        }
        Err(self.raise_missing_error("prefix")?)
    }

    fn gen_type(&self) -> Result<Str, Error> {
        if let Some(type_name) = &self.ty {
            return Ok(type_name.clone());
        }
        Err(self.raise_missing_error("type name")?)
    }

    fn gen_idx(&self) -> Result<OptIndex, Error> {
        if let Some(index) = self.idx.as_ref() {
            return Ok(index.clone());
        }
        Err(Error::con_missing_index(self.gen_name()?, self.gen_type()?))
    }

    fn gen_alias(&self) -> Result<Vec<(Str, Str)>, Error> {
        let mut ret = vec![];

        for alias in self.alias.iter() {
            let mut found_prefix = false;

            for prefix in self.sp_pre.iter() {
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
        if let Some(optional) = self.opt {
            return Ok(optional);
        }
        Err(self.raise_missing_error("optional")?)
    }

    fn gen_opt_help(&self, deactivate_style: bool) -> Result<OptHelp, Error> {
        let mut ret = self.help.clone();

        if ret.hint().is_empty() {
            let mut names = vec![String::default()];

            // add prefix
            if let Some(prefix) = self.prefix() {
                names[0] += prefix.as_str();
            }
            // add deactivate style
            if deactivate_style {
                names[0] += "/";
            }
            // add name
            names[0] += self.gen_name()?.as_ref();

            // add alias
            if let Some(alias_vec) = self.alias() {
                for alias in alias_vec {
                    if deactivate_style {
                        for prefix in self.sp_prefix() {
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

            if let Some(index) = &self.idx {
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

    fn gen_deactivate(&self) -> Result<bool, Error> {
        if let Some(deactivate_style) = self.deact {
            return Ok(deactivate_style);
        }
        Err(self.raise_missing_error("deactivate_style")?)
    }

    fn gen_policy(&self) -> Option<(ValPolicy, ValType)> {
        self.policy.clone()
    }

    fn take_uid(&mut self) -> Uid {
        self.uid
    }

    fn take_name(&mut self) -> Option<Str> {
        self.name.take()
    }

    fn take_prefix(&mut self) -> Option<Str> {
        self.pre.take()
    }

    fn take_type(&mut self) -> Option<Str> {
        self.ty.take()
    }

    fn take_idx(&mut self) -> Option<OptIndex> {
        self.idx.take()
    }

    fn take_alias(&mut self) -> Option<Vec<Str>> {
        Some(std::mem::take(&mut self.alias))
    }

    fn take_optional(&mut self) -> Option<bool> {
        self.opt.take()
    }

    fn take_opt_help(&mut self) -> OptHelp {
        std::mem::take(&mut self.help)
    }

    fn take_deactivate(&mut self) -> Option<bool> {
        self.deact.take()
    }

    fn take_policy(&mut self) -> Option<(ValPolicy, ValType)> {
        self.policy.take()
    }
}
