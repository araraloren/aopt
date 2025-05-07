use syn::Ident;

use super::Kind;
use super::Style;
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
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, Style)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            "val" => Ok((Self::Val, Style::Value)),
            "action" => Ok((Self::Action, Style::Value)),
            "force" => Ok((Self::Force, Style::Value)),
            "ctor" => Ok((Self::Ctor, Style::Value)),
            "index" => Ok((Self::Index, Style::Value)),
            "style" => Ok((Self::Style, Style::Value)),
            "igname" => Ok((Self::IgName, Style::Value)),
            "igalias" => Ok((Self::IgAlias, Style::Value)),
            "igindex" => Ok((Self::IgIndex, Style::Value)),
            "valid" => Ok((Self::Valid, Style::Value)),
            "init" => Ok((Self::Init, Style::Value)),
            "ty" => Ok((Self::Type, Style::Value)),
            "map" => Ok((Self::Map, Style::Value)),
            "mutable" => Ok((Self::Mutable, Style::Value)),
            "tweak" => Ok((Self::Tweak, Style::Value)),
            "fill" => Ok((Self::Fill, Style::Value)),
            "override" => Ok((Self::Override, Style::Flag)),
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
