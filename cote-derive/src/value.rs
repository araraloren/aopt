use std::iter::FromIterator;

use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn::parenthesized;
use syn::parse::Parse;
use syn::token::Paren;
use syn::Expr;
use syn::Lit;
use syn::LitInt;
use syn::Token;

use crate::error;

#[derive(Debug, Clone)]
pub enum Value {
    Literal(Lit),

    Expr(Expr),

    Call(Vec<Expr>),

    Null,
}

impl Value {
    pub fn split_call_args(self, span: Span) -> syn::Result<(Expr, Self)> {
        if let Value::Call(mut args) = self {
            if !args.is_empty() {
                let method_args = args.split_off(1);

                return Ok((args.pop().unwrap(), Self::Call(method_args)));
            }
        }
        Err(error(
            span,
            "You must specify the context variable name for raw method call".to_owned(),
        ))
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Literal(t) => t.to_tokens(tokens),
            Self::Expr(t) => t.to_tokens(tokens),
            Self::Call(t) => {
                let t = quote!(#(#t),*);
                t.to_tokens(tokens)
            }
            Self::Null => {}
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![=]) {
            let assign_token = input.parse::<Token![=]>()?;

            // not seems like a range
            if input.peek(Lit)
                && !(input.peek(LitInt) && input.peek2(Token![.]) && input.peek3(Token![.]))
            {
                Ok(Self::Literal(input.parse()?))
            } else {
                match input.parse::<Expr>() {
                    Ok(expr) => Ok(Value::Expr(expr)),
                    Err(e) => Err(error(
                        assign_token,
                        format!(
                            "excepted `string literal` or `expression` after `=`: {:?}",
                            e
                        ),
                    )),
                }
            }
        } else if input.peek(Paren) {
            // `name(...)` attributes
            let nested;

            parenthesized!(nested in input);

            let method_args = nested.parse_terminated(Expr::parse, Token![,])?;

            Ok(Self::Call(Vec::from_iter(method_args)))
        } else {
            Err(error(
                input.span(),
                "invalid configuration value".to_owned(),
            ))
        }
    }
}
