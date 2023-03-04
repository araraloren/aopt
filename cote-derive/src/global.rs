pub(crate) use crate::value::CfgValue;

use proc_macro2::Ident;
use proc_macro_error::abort;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::Attribute;
use syn::Token;

pub(crate) trait Attr {
    fn cfg_kind(&self) -> CfgKind;

    fn cfg_value(&self) -> &CfgValue;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum CfgKind {
    ParserPolicy,

    ParserOn,

    ParserHelp,

    ParserHelpWidth,

    ParserUsageWidth,

    ParserName,

    ParserHead,

    ParserFoot,

    SubPolicy,

    SubName,

    SubAlias,

    SubHint,

    SubHelp,

    SubRef,

    SubMut,

    OptHint,

    OptHelp,

    OptName,

    OptValue,

    OptValues,

    OptAlias,

    OptAction,

    OptIndex,

    OptValidator,

    OptForce,

    OptNoForce,

    OptOn,

    OptFallback,

    OptThen,

    OptRef,

    OptMut,

    OptNoDelay,
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
            "policy" => CfgKind::ParserPolicy,
            "help" => CfgKind::ParserHelp,
            "width" => CfgKind::ParserHelpWidth,
            "usagew" => CfgKind::ParserUsageWidth,
            "head" => CfgKind::ParserHead,
            "foot" => CfgKind::ParserFoot,
            "on" => CfgKind::ParserOn,
            "name" => CfgKind::ParserName,
            _ => {
                abort! {
                    ident, "invalid configuration name in parser(...): {:?}", cfg_kind
                }
            }
        };

        match cfg_kind {
            CfgKind::ParserHelp => Ok(Self {
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

#[derive(Debug, Clone)]
pub(crate) struct ArgCfg {
    pub kind: CfgKind,

    pub value: CfgValue,
}

impl Attr for ArgCfg {
    fn cfg_kind(&self) -> CfgKind {
        self.kind
    }

    fn cfg_value(&self) -> &CfgValue {
        &self.value
    }
}

impl Parse for ArgCfg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let cfg_kind = ident.to_string();
        let cfg_kind = match cfg_kind.as_str() {
            "name" => CfgKind::OptName,
            "hint" => CfgKind::OptHint,
            "help" => CfgKind::OptHelp,
            "value" => CfgKind::OptValue,
            "values" => CfgKind::OptValues,
            "alias" => CfgKind::OptAlias,
            "index" => CfgKind::OptIndex,
            "action" => CfgKind::OptAction,
            "valid" => CfgKind::OptValidator,
            "on" => CfgKind::OptOn,
            "fallback" => CfgKind::OptFallback,
            "then" => CfgKind::OptThen,
            "ref" => CfgKind::OptRef,
            "mut" => CfgKind::OptMut,
            "force" => CfgKind::OptForce,
            "noforce" => CfgKind::OptNoForce,
            "nodelay" => CfgKind::OptNoDelay,
            _ => {
                abort! {
                    ident, "invalid configuration name in arg(...): {:?}", cfg_kind
                }
            }
        };

        match cfg_kind {
            CfgKind::OptForce
            | CfgKind::OptNoForce
            | CfgKind::OptNoDelay
            | CfgKind::OptRef
            | CfgKind::OptMut => Ok(Self {
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

impl From<ArgCfg> for FieldCfg {
    fn from(value: ArgCfg) -> Self {
        FieldCfg {
            kind: value.kind,
            value: value.value,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SubCfg {
    pub kind: CfgKind,

    pub value: CfgValue,
}

impl Attr for SubCfg {
    fn cfg_kind(&self) -> CfgKind {
        self.kind
    }

    fn cfg_value(&self) -> &CfgValue {
        &self.value
    }
}

impl Parse for SubCfg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let cfg_kind = ident.to_string();
        let cfg_kind = match cfg_kind.as_str() {
            "policy" => CfgKind::SubPolicy,
            "name" => CfgKind::SubName,
            "alias" => CfgKind::SubAlias,
            "ref" => CfgKind::SubRef,
            "mut" => CfgKind::SubMut,
            "hint" => CfgKind::SubHint,
            "help" => CfgKind::SubHelp,
            _ => {
                abort! {
                    ident, "invalid configuration name in sub(...): {:?}", cfg_kind
                }
            }
        };

        Ok(Self {
            kind: cfg_kind,
            value: input.parse()?,
        })
    }
}

impl From<SubCfg> for FieldCfg {
    fn from(value: SubCfg) -> Self {
        FieldCfg {
            kind: value.kind,
            value: value.value,
        }
    }
}

#[derive(Debug)]
pub(crate) struct FieldCfg {
    pub kind: CfgKind,

    pub value: CfgValue,
}

impl Parse for FieldCfg {
    fn parse(_input: syn::parse::ParseStream) -> syn::Result<Self> {
        unimplemented!("not implement")
    }
}

impl Attr for FieldCfg {
    fn cfg_kind(&self) -> CfgKind {
        self.kind
    }

    fn cfg_value(&self) -> &CfgValue {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Configurations<T> {
    pub cfgs: Vec<T>,
}

impl<T: Attr> Configurations<T> {
    pub fn find_cfg(&self, kind: CfgKind) -> Option<&T> {
        self.cfgs.iter().find(|v| v.cfg_kind() == kind)
    }
}

impl<T: Parse> Configurations<T> {
    pub fn parse_attrs(attrs: &[Attribute], name: &str) -> Self {
        let attrs = attrs.iter().filter(|v| v.path.is_ident(name));
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

        Self {
            cfgs: cfgs.flatten().collect::<Vec<T>>(),
        }
    }
}
