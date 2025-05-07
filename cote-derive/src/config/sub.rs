use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, Path};

use super::Kind;
use super::Style;

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

    PrePolicy,

    MethodCall(String),
}

impl Kind for SubKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, Style)> {
        let path: Path = input.parse()?;

        if let Some(ident) = path.get_ident() {
            let kind_str = ident.to_string();

            Ok(match kind_str.as_str() {
                "policy" => (Self::Policy, Style::Value),
                "name" => (Self::Name, Style::Value),
                "alias" => (Self::Alias, Style::Value),
                "hint" => (Self::Hint, Style::Value),
                "help" => (Self::Help, Style::Value),
                "head" => (Self::Head, Style::Value),
                "foot" => (Self::Foot, Style::Value),
                "refopt" => (Self::Ref, Style::Flag),
                "mutopt" => (Self::Mut, Style::Flag),
                "force" => (Self::Force, Style::True),
                "prepolicy" => (Self::PrePolicy, Style::True),
                method => (Self::MethodCall(method.to_owned()), Style::Value),
            })
        } else {
            let method = path.to_token_stream().to_string();
            let method = method.replace(char::is_whitespace, "");

            Ok((Self::MethodCall(method), Style::Value))
        }
    }
}

impl SubKind {
    pub fn simple(&self, ident: &Ident, val: &TokenStream) -> syn::Result<TokenStream> {
        match self {
            SubKind::Name => Ok(quote! {
                cote::prelude::ConfigValue::set_name(&mut #ident, #val);
            }),
            SubKind::Hint => Ok(quote! {
                cote::prelude::ConfigValue::set_hint(&mut #ident, #val);
            }),

            SubKind::Help => Ok(quote! {
                cote::prelude::ConfigValue::set_help(&mut #ident, #val);
            }),
            SubKind::Alias => Ok(quote! {
                cote::prelude::ConfigValue::add_alias(&mut #ident, #val);
            }),
            SubKind::Force => Ok(quote! {
                cote::prelude::ConfigValue::set_force(&mut #ident, #val);
            }),
            _ => Err(crate::error(ident.span(), "")),
        }
    }
}
