use proc_macro_error::abort;
use syn::Ident;

use super::Kind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueKind {
    Map,

    IgCase,

    MapStr,
}

impl Kind for ValueKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            "mapstr" => Ok((Self::MapStr, true)),
            "map" => Ok((Self::Map, true)),
            "igcase" => Ok((Self::IgCase, false)),
            _ => {
                abort! {
                    input.span(),
                    "Unknow configuration name {} in attribute rawvalparser", option
                }
            }
        }
    }
}
