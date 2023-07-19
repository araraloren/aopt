use std::iter::FromIterator;

use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::parenthesized;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::Expr;
use syn::Lit;
use syn::LitInt;
use syn::Token;

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
            if let Some(variable) = args.pop() {
                return Ok((variable, Self::Call(args)));
            }
        }
        abort! {
            span,
            "You must specify the context variable name for raw method call"
        }
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
                    Err(e) => abort! {
                        assign_token,
                        "excepted `string literal` or `expression` after `=`: {:?}", e
                    },
                }
            }
        } else if input.peek(Paren) {
            // `name(...)` attributes
            let nested;

            parenthesized!(nested in input);

            let method_args: Punctuated<_, Token![,]> = nested.parse_terminated(Expr::parse)?;

            Ok(Self::Call(Vec::from_iter(method_args)))
        } else {
            abort!(input.span(), "invalid configuration value")
        }
    }
}
