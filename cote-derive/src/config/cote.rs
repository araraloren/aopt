use quote::ToTokens;
use syn::Path;

use super::Kind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoteKind {
    Policy,

    Name,

    Help,

    Head,

    Foot,

    HelpWidth,

    UsageWidth,

    AbortHelp,

    On,

    Fallback,

    Then,

    Strict,

    Combine,

    EmbeddedPlus,

    Flag,

    Overload,

    MethodCall(String),
}

impl Kind for CoteKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let path: Path = input.parse()?;

        if let Some(ident) = path.get_ident() {
            let kind_str = ident.to_string();

            Ok(match kind_str.as_str() {
                "policy" => (Self::Policy, true),
                "name" => (Self::Name, true),
                "help" => (Self::Help, false),
                "head" => (Self::Head, true),
                "foot" => (Self::Foot, true),
                "width" => (Self::HelpWidth, true),
                "usagew" => (Self::UsageWidth, true),
                "aborthelp" => (Self::AbortHelp, false),
                "on" => (Self::On, true),
                "fallback" => (Self::Fallback, true),
                "then" => (Self::Then, true),
                "strict" => (Self::Strict, true),
                "combine" => (Self::Combine, false),
                "embedded" => (Self::EmbeddedPlus, false),
                "flag" => (Self::Flag, false),
                "overload" => (Self::Overload, false),
                method => (Self::MethodCall(method.to_owned()), true),
            })
        } else {
            let method = path.to_token_stream().to_string();
            let method = method.replace(char::is_whitespace, "");

            Ok((Self::MethodCall(method), true))
        }
    }
}
