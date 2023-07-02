use quote::ToTokens;
use syn::Path;

use super::Kind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubKind {
    Policy,

    Name,

    Alias,

    Hint,

    Help,

    Head,

    Foot,

    Ref,

    Mut,

    Force,

    MethodCall(String),
}

impl Kind for SubKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let path: Path = input.parse()?;

        if let Some(ident) = path.get_ident() {
            let kind_str = ident.to_string();

            Ok(match kind_str.as_str() {
                "policy" => (Self::Policy, true),
                "name" => (Self::Name, true),
                "alias" => (Self::Alias, true),
                "hint" => (Self::Hint, true),
                "help" => (Self::Help, true),
                "head" => (Self::Head, true),
                "foot" => (Self::Foot, true),
                "refopt" => (Self::Ref, false),
                "mutopt" => (Self::Mut, false),
                "force" => (Self::Force, true),
                method => (Self::MethodCall(method.to_owned()), true),
            })
        } else {
            let method = path.to_token_stream().to_string();
            let method = method.replace(char::is_whitespace, "");

            Ok((Self::MethodCall(method), true))
        }
    }
}
