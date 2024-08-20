use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, Path};

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

    Fetch,

    Append,

    Count,

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
                "fetch" => (Self::Fetch, true),
                "append" => (Self::Append, false),
                "count" => (Self::Count, false),
                method => (Self::MethodCall(method.to_owned()), true),
            })
        } else {
            let method = path.to_token_stream().to_string();
            let method = method.replace(char::is_whitespace, "");

            Ok((Self::MethodCall(method), true))
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
