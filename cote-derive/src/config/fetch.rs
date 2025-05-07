use super::Kind;
use super::Style;
use crate::error;

use syn::Ident;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FetchKind {
    Inner,

    Map,

    Handle,
}

impl Kind for FetchKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, Style)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            "inner" => Ok((Self::Inner, Style::Value)),
            "map" => Ok((Self::Map, Style::Value)),
            "handle" => Ok((Self::Handle, Style::Value)),
            _ => Err(error(
                input.span(),
                format!(
                    "unknown configuration name `{}` in attribute fetch",
                    option.as_str()
                ),
            )),
        }
    }
}
