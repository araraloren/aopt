pub(crate) use crate::value::CfgValue;

use proc_macro2::Ident;
use proc_macro_error::abort;
use syn::{parse::Parse, punctuated::Punctuated, Attribute, Token};

pub(crate) trait Attr {
    fn cfg_kind(&self) -> CfgKind;

    fn cfg_value(&self) -> &CfgValue;
}

pub(crate) trait ConfigCheck {
    fn has_policy(&self) -> bool {
        false
    }

    fn check(&self, has_policy: bool) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum CfgKind {
    Policy,

    Hint,

    Help,

    Author,

    Version,

    Head,

    Foot,

    Name,

    Value,

    Values,

    Alias,

    Action,

    Index,

    Validator,

    On,
}

#[derive(Debug, Clone)]
pub(crate) struct GlobalCfg {
    pub kind: CfgKind,

    pub value: CfgValue,
}

impl Attr for GlobalCfg {
    fn cfg_kind(&self) -> CfgKind {
        self.kind
    }

    fn cfg_value(&self) -> &CfgValue {
        &self.value
    }
}

impl Parse for GlobalCfg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let cfg_kind = ident.to_string();
        let cfg_kind = match cfg_kind.as_str() {
            "policy" => CfgKind::Policy,
            "help" => CfgKind::Help,
            "author" => CfgKind::Author,
            "version" => CfgKind::Version,
            "head" => CfgKind::Head,
            "foot" => CfgKind::Foot,
            _ => {
                abort! {
                    ident, "invalid configuration name in cote(...): {:?}", cfg_kind
                }
            }
        };

        match cfg_kind {
            CfgKind::Help | CfgKind::Author | CfgKind::Version => Ok(Self {
                kind: cfg_kind,
                value: CfgValue::Null,
            }),
            _ => Ok(Self {
                kind: cfg_kind,
                value: input.parse()?,
            }),
        }
    }
}

impl ConfigCheck for GlobalCfg {
    fn check(&self, _: bool) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FieldCfg {
    pub kind: CfgKind,

    pub value: CfgValue,
}

impl Attr for FieldCfg {
    fn cfg_kind(&self) -> CfgKind {
        self.kind
    }

    fn cfg_value(&self) -> &CfgValue {
        &self.value
    }
}

impl Parse for FieldCfg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let cfg_kind = ident.to_string();
        let cfg_kind = match cfg_kind.as_str() {
            "policy" => CfgKind::Policy,
            "name" => CfgKind::Name,
            "hint" => CfgKind::Hint,
            "help" => CfgKind::Help,
            "value" => CfgKind::Value,
            "values" => CfgKind::Values,
            "alias" => CfgKind::Alias,
            "index" => CfgKind::Index,
            "action" => CfgKind::Action,
            "valid" => CfgKind::Validator,
            "on" => CfgKind::On,
            _ => {
                abort! {
                    ident, "invalid configuration name in cote(...): {:?}", cfg_kind
                }
            }
        };

        Ok(Self {
            kind: cfg_kind,
            value: input.parse()?,
        })
    }
}

impl ConfigCheck for FieldCfg {
    fn check(&self, has_policy: bool) -> bool {
        match self.kind {
            CfgKind::Policy => has_policy,
            CfgKind::Name | CfgKind::Alias => true,
            _ => !has_policy,
        }
    }

    fn has_policy(&self) -> bool {
        matches!(self.kind, CfgKind::Policy)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Configurations<T: Parse> {
    pub cfgs: Vec<T>,
}

impl<T: Parse + Attr> Configurations<T> {
    pub fn find_cfg(&self, kind: CfgKind) -> Option<&T> {
        self.cfgs.iter().find(|v| v.cfg_kind() == kind)
    }
}

impl<T: Parse + ConfigCheck> Configurations<T> {
    pub fn parse_attrs(ident: Option<&Ident>, attrs: &[Attribute]) -> Self {
        let attrs = attrs.iter().filter(Self::cote_filter);
        let cfgs = attrs.map(|attr| {
            attr.parse_args_with(Punctuated::<T, Token![,]>::parse_terminated)
                .map(|res| res.into_iter())
                .unwrap_or_else(|e| {
                    abort! {
                        attr,
                        "can not parsing cote attributes: {:?}", e
                    }
                })
        });
        let cfgs = cfgs.flatten().collect::<Vec<T>>();
        let has_policy = cfgs.iter().any(|v| v.has_policy());

        for cfg in cfgs.iter() {
            if !cfg.check(has_policy) {
                abort! {
                    ident,
                    "can not have attribute except `name` and `alias` if `policy` set"
                }
            }
        }

        Self { cfgs }
    }

    fn cote_filter(attr: &&Attribute) -> bool {
        attr.path.is_ident("cote")
    }
}
