use proc_macro2::Ident;
use proc_macro_error::abort;

use super::Kind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArgKind {
    Name,

    Hint,

    Help,

    Value,

    Values,

    Alias,

    Index,

    Force,

    NoForce,

    Action,

    Validator,

    On,

    Fallback,

    Then,

    Ref,

    Mut,

    NoDelay,
}

impl Kind for ArgKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let kind_str = ident.to_string();

        Ok(match kind_str.as_str() {
            "name" => (Self::Name, true),
            "hint" => (Self::Hint, true),
            "help" => (Self::Help, true),
            "value" => (Self::Value, true),
            "values" => (Self::Values, true),
            "alias" => (Self::Alias, true),
            "index" => (Self::Index, true),
            "force" => (Self::Force, false),
            "noforce" => (Self::NoForce, false),
            "action" => (Self::Action, true),
            "valid" => (Self::Validator, true),
            "on" => (Self::On, true),
            "fallback" => (Self::Fallback, true),
            "then" => (Self::Then, true),
            "refopt" => (Self::Ref, false),
            "mutopt" => (Self::Mut, false),
            "nodelay" => (Self::NoDelay, false),
            _ => {
                abort! {
                    ident,
                    "invalid configuration name `{}` in `arg`", kind_str
                }
            }
        })
    }
}
