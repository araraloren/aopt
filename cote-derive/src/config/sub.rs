use proc_macro2::Ident;

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

    RawCall(String),
}

impl Kind for SubKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
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
            call => (Self::RawCall(call.to_owned()), true),
        })
    }
}
