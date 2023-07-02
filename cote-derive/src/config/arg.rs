use quote::ToTokens;
use syn::Path;

use super::Kind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArgKind {
    Name,

    Type,

    Hint,

    Help,

    Value,

    Values,

    Alias,

    Index,

    Force,

    Action,

    Validator,

    On,

    Fallback,

    Then,

    NoDelay,

    MethodCall(String),
}

impl Kind for ArgKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let path: Path = input.parse()?;

        if let Some(ident) = path.get_ident() {
            let kind_str = ident.to_string();

            Ok(match kind_str.as_str() {
                "name" => (Self::Name, true),
                "ty" => (Self::Type, true),
                "hint" => (Self::Hint, true),
                "help" => (Self::Help, true),
                "value" => (Self::Value, true),
                "values" => (Self::Values, true),
                "alias" => (Self::Alias, true),
                "index" => (Self::Index, true),
                "force" => (Self::Force, true),
                "action" => (Self::Action, true),
                "valid" => (Self::Validator, true),
                "on" => (Self::On, true),
                "fallback" => (Self::Fallback, true),
                "then" => (Self::Then, true),
                "nodelay" => (Self::NoDelay, false),
                method => (Self::MethodCall(method.to_owned()), true),
            })
        } else {
            let method = path.to_token_stream().to_string();
            let method = method.replace(char::is_whitespace, "");

            Ok((Self::MethodCall(method), true))
        }
    }
}
