use quote::ToTokens;
use syn::Path;

use super::Kind;
use super::Style;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoteKind {
    Policy,

    Name,

    Help,

    HelpOpt,

    Head,

    Foot,

    HelpWidth,

    UsageWidth,

    AbortHelp,

    NotExit,

    On,

    Fallback,

    Then,

    Strict,

    Combine,

    EmbeddedPlus,

    Flag,

    Overload,

    PrePolicy,

    ShellCompletion,

    MethodCall(String),
}

impl Kind for CoteKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, Style)> {
        let path: Path = input.parse()?;

        if let Some(ident) = path.get_ident() {
            let kind_str = ident.to_string();

            Ok(match kind_str.as_str() {
                "policy" => (Self::Policy, Style::Value),
                "name" => (Self::Name, Style::Value),
                "help" => (Self::Help, Style::Flag),
                "helpopt" => (Self::HelpOpt, Style::Value),
                "head" => (Self::Head, Style::Value),
                "foot" => (Self::Foot, Style::Value),
                "width" => (Self::HelpWidth, Style::Value),
                "usagew" => (Self::UsageWidth, Style::Value),
                "aborthelp" => (Self::AbortHelp, Style::Flag),
                "notexit" => (Self::NotExit, Style::Flag),
                "on" => (Self::On, Style::Value),
                "fallback" => (Self::Fallback, Style::Value),
                "then" => (Self::Then, Style::Value),
                "strict" => (Self::Strict, Style::True),
                "combine" => (Self::Combine, Style::Flag),
                "embedded" => (Self::EmbeddedPlus, Style::Flag),
                "flag" => (Self::Flag, Style::Flag),
                "overload" => (Self::Overload, Style::True),
                "prepolicy" => (Self::PrePolicy, Style::True),
                "shellcomp" => (Self::ShellCompletion, Style::True),
                method => (Self::MethodCall(method.to_owned()), Style::Value),
            })
        } else {
            let method = path.to_token_stream().to_string();
            let method = method.replace(char::is_whitespace, "");

            Ok((Self::MethodCall(method), Style::Value))
        }
    }
}
