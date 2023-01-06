pub(crate) use crate::value::CfgValue;

use proc_macro2::Ident;
use proc_macro_error::abort;
use syn::{parse::Parse, punctuated::Punctuated, Attribute, Token};

pub(crate) trait Attr {
    fn cfg_kind(&self) -> CfgKind;

    fn cfg_value(&self) -> &CfgValue;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum CfgKind {
    Policy,

    Name,

    Type,

    Hint,

    // Help, // can be set by comment
    Value,

    Values,

    Force, // !

    Index,

    Alias,

    Action, // determind how to store value

    Assoc, // determind which type value store in default handler

    On,

    Fallback,

    Skip,
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
            "ty" => CfgKind::Type,
            "hint" => CfgKind::Hint,
            "val" => CfgKind::Value,
            "vals" => CfgKind::Values,
            "force" => CfgKind::Force,
            "index" => CfgKind::Index,
            "alias" => CfgKind::Alias,
            "act" => CfgKind::Action,
            "assoc" => CfgKind::Assoc,
            "on" => CfgKind::On,
            "fallback" => CfgKind::Fallback,
            "skip" => CfgKind::Skip,
            _ => {
                abort! {
                    ident, "invalid configuration name in cote(...): {:?}", cfg_kind
                }
            }
        };

        match cfg_kind {
            CfgKind::Skip => Ok(Self {
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

#[derive(Debug, Clone, Default)]
pub(crate) struct Configurations<T: Parse> {
    cfgs: Vec<T>,
}

impl<T: Parse + Attr> Configurations<T> {
    pub fn find_cfg(&self, kind: CfgKind) -> Option<&T> {
        self.cfgs.iter().find(|v| v.cfg_kind() == kind)
    }
}

impl<T: Parse> Configurations<T> {
    pub fn parse_attrs(attrs: &[Attribute]) -> Self {
        Self {
            cfgs: attrs
                .iter()
                .filter(Self::cote_filter)
                .map(|attr| {
                    attr.parse_args_with(Punctuated::<T, Token![,]>::parse_terminated)
                        .map(|res| res.into_iter())
                        .unwrap_or_else(|e| {
                            abort! {
                                attr,
                                "can not parsing cote attributes: {:?}", e
                            }
                        })
                })
                .flatten()
                .collect::<Vec<T>>(),
        }
    }

    fn cote_filter(attr: &&Attribute) -> bool {
        attr.path.is_ident("cote")
    }
}
