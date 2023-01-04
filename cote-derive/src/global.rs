pub(crate) use crate::value::CfgValue;

use proc_macro2::Ident;
use proc_macro_error::abort;
use syn::{parse::Parse, punctuated::Punctuated, Attribute, Token, Lit, Expr, token::Paren, parenthesized};

#[derive(Debug, Clone)]
pub enum CfgKind {
    Policy,
}

#[derive(Debug, Clone)]
pub(crate) struct GlobalCfg {
    pub kind: CfgKind,

    pub value: CfgValue,
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

        Ok(GlobalCfg { kind: cfg_kind, value: input.parse()? })
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GlobalCfgs {
    cfgs: Vec<GlobalCfg>,
}

impl GlobalCfgs {
    pub fn parse_attrs(attrs: &[Attribute]) -> Self {
        Self {
            cfgs: attrs
                .iter()
                .filter(Self::cote_filter)
                .map(|attr| {
                    attr.parse_args_with(Punctuated::<GlobalCfg, Token![,]>::parse_terminated)
                        .map(|res| res.into_iter())
                        .unwrap_or_else(|e| {
                            abort! {
                                attr,
                                "can not parsing cote attributes: {:?}", e
                            }
                        })
                })
                .flatten()
                .collect::<Vec<GlobalCfg>>(),
        }
    }

    fn cote_filter(attr: &&Attribute) -> bool {
        attr.path.is_ident("cote")
    }
}
