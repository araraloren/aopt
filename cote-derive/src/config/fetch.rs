use super::Kind;
use crate::error;

use syn::Ident;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FetchKind {
    Inner,

    Map,

    Scalar,

    Vector,
}

impl Kind for FetchKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            "inner" => Ok((Self::Inner, true)),
            "map" => Ok((Self::Map, true)),
            "scalar" => Ok((Self::Scalar, true)),
            "vector" => Ok((Self::Vector, true)),
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
