use proc_macro_error::abort;
use syn::Ident;

use super::Kind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueKind {
    Forward,

    Map,

    MapRaw,

    IgCase,

    MapStr,

    Name,

    Alias,
}

impl Kind for ValueKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            "forward" => Ok((Self::Forward, true)),
            "map" => Ok((Self::Map, true)),
            "mapraw" => Ok((Self::MapRaw, true)),
            "mapstr" => Ok((Self::MapStr, true)),
            "igcase" => Ok((Self::IgCase, false)),
            "name" => Ok((Self::Name, true)),
            "alias" => Ok((Self::Alias, true)),
            _ => {
                abort! {
                    input.span(),
                    "Unknow configuration name {} in attribute rawvalparser", option
                }
            }
        }
    }
}
