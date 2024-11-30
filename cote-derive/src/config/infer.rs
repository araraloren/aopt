use syn::Ident;

use super::Kind;
use crate::error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InferKind {
    Val,

    Action,

    Force,

    Ctor,

    Index,

    Style,

    IgName,

    IgAlias,

    IgIndex,

    Valid,

    Init,

    Type,

    Map,

    Mutable,

    Tweak,

    Fill,

    Override,
}

impl Kind for InferKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            "val" => Ok((Self::Val, true)),
            "action" => Ok((Self::Action, true)),
            "force" => Ok((Self::Force, false)),
            "ctor" => Ok((Self::Ctor, true)),
            "index" => Ok((Self::Index, true)),
            "style" => Ok((Self::Style, true)),
            "igname" => Ok((Self::IgName, false)),
            "igalias" => Ok((Self::IgAlias, false)),
            "igindex" => Ok((Self::IgIndex, false)),
            "valid" => Ok((Self::Valid, true)),
            "init" => Ok((Self::Init, true)),
            "ty" => Ok((Self::Type, true)),
            "map" => Ok((Self::Map, true)),
            "mutable" => Ok((Self::Mutable, true)),
            "tweak" => Ok((Self::Tweak, true)),
            "fill" => Ok((Self::Fill, true)),
            "override" => Ok((Self::Override, false)),
            _ => Err(error(
                input.span(),
                format!(
                    "unknown configuration name `{}` in attribute infer",
                    option.as_str()
                ),
            )),
        }
    }
}
