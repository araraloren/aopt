use proc_macro2::Ident;
use proc_macro_error::abort;

use super::Kind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoteKind {
    Policy,

    Name,

    Hint,

    Help,

    Head,

    Foot,

    HelpWidth,

    UsageWidth,

    AbortHelp,

    Ref,

    Mut,

    On,

    Combine,

    EmbeddedPlus,
}

impl Kind for CoteKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let kind_str = ident.to_string();

        Ok(match kind_str.as_str() {
            "policy" => (Self::Policy, true),
            "name" => (Self::Name, true),
            "hint" => (Self::Hint, true),
            "help" => (Self::Help, true),
            "head" => (Self::Head, true),
            "foot" => (Self::Foot, true),
            "width" => (Self::HelpWidth, true),
            "usagew" => (Self::UsageWidth, true),
            "aborthelp" => (Self::AbortHelp, false),
            "refopt" => (Self::Ref, false),
            "mutopt" => (Self::Mut, false),
            "on" => (Self::On, true),
            "combine" => (Self::Combine, false),
            "embedded" => (Self::EmbeddedPlus, false),
            _ => {
                abort! {
                    ident,
                    "invalid configuration name `{}` in `arg`", kind_str
                }
            }
        })
    }
}
