use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, Path};

use super::Kind;
use super::Style;

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

    Fetch,

    Append,

    Count,

    MethodCall(String),
}

impl Kind for ArgKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, Style)> {
        let path: Path = input.parse()?;

        if let Some(ident) = path.get_ident() {
            let kind_str = ident.to_string();

            Ok(match kind_str.as_str() {
                "name" => (Self::Name, Style::Value),
                "ty" => (Self::Type, Style::Value),
                "hint" => (Self::Hint, Style::Value),
                "help" => (Self::Help, Style::Value),
                "value" => (Self::Value, Style::Value),
                "values" => (Self::Values, Style::Value),
                "alias" => (Self::Alias, Style::Value),
                "index" => (Self::Index, Style::Value),
                "force" => (Self::Force, Style::Value),
                "action" => (Self::Action, Style::Value),
                "valid" => (Self::Validator, Style::Value),
                "on" => (Self::On, Style::Value),
                "fallback" => (Self::Fallback, Style::Value),
                "then" => (Self::Then, Style::Value),
                "nodelay" => (Self::NoDelay, Style::Flag),
                "fetch" => (Self::Fetch, Style::Value),
                "append" => (Self::Append, Style::Flag),
                "count" => (Self::Count, Style::Flag),
                method => (Self::MethodCall(method.to_owned()), Style::Value),
            })
        } else {
            let method = path.to_token_stream().to_string();
            let method = method.replace(char::is_whitespace, "");

            Ok((Self::MethodCall(method), Style::Value))
        }
    }
}

impl ArgKind {
    pub fn simple(&self, ident: &Ident, val: TokenStream) -> syn::Result<TokenStream> {
        match self {
            ArgKind::Name => Ok(quote! {
                cote::prelude::ConfigValue::set_name(&mut #ident, #val);
            }),
            ArgKind::Hint => Ok(quote! {
                cote::prelude::ConfigValue::set_hint(&mut #ident, #val);
            }),

            ArgKind::Help => Ok(quote! {
                cote::prelude::ConfigValue::set_help(&mut #ident, #val);
            }),
            ArgKind::Value => Ok(quote! {
                cote::prelude::ConfigValue::set_initializer(&mut #ident, cote::prelude::ValInitializer::new_value(#val));
            }),
            ArgKind::Values => Ok(quote! {
                cote::prelude::ConfigValue::set_initializer(&mut #ident, cote::prelude::ValInitializer::new_values(#val));
            }),
            ArgKind::Alias => Ok(quote! {
                cote::prelude::ConfigValue::add_alias(&mut #ident, #val);
            }),
            ArgKind::Index => Ok(quote! {
                cote::prelude::ConfigValue::set_index(&mut #ident, <cote::prelude::Index as std::convert::TryFrom::<_>>::try_from(#val)?);
            }),
            ArgKind::Force => Ok(quote! {
                cote::prelude::ConfigValue::set_force(&mut #ident, #val);
            }),
            ArgKind::Action => Ok(quote! {
                cote::prelude::ConfigValue::set_action(&mut #ident, #val);
            }),
            ArgKind::Validator => Ok(quote! {
                cote::prelude::ConfigValue::set_storer(&mut #ident, #val);
            }),
            ArgKind::Append => Ok(quote! {
                cote::prelude::ConfigValue::set_action(&mut #ident, cote::prelude::Action::App);
            }),
            ArgKind::Count => Ok(quote! {
                cote::prelude::ConfigValue::set_action(&mut #ident, cote::prelude::Action::Cnt);
            }),
            _ => Err(crate::error(ident.span(), "")),
        }
    }
}
