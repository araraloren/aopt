use syn::Ident;

use super::Kind;
use super::Style;
use crate::error;

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
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, Style)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            "forward" => Ok((Self::Forward, Style::Value)),
            "map" => Ok((Self::Map, Style::Value)),
            "mapraw" => Ok((Self::MapRaw, Style::Value)),
            "mapstr" => Ok((Self::MapStr, Style::Value)),
            "igcase" => Ok((Self::IgCase, Style::Flag)),
            "name" => Ok((Self::Name, Style::Value)),
            "alias" => Ok((Self::Alias, Style::Value)),
            _ => Err(error(
                input.span(),
                format!(
                    "unknown configuration name `{}` in attribute rawvalparser",
                    option.as_str()
                ),
            )),
        }
    }
}
