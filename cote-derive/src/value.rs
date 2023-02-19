use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, punctuated::Punctuated, token::Paren, Expr, Lit, Token};

#[derive(Debug, Clone)]
pub(crate) enum CfgValue {
    Literal(Lit),

    Expr(Expr),

    Call(Vec<Expr>),

    Null,
}

impl ToTokens for CfgValue {
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

impl Default for CfgValue {
    fn default() -> Self {
        Self::Null
    }
}

impl Parse for CfgValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![=]) {
            let assign_token = input.parse::<Token![=]>()?;

            if input.peek(Lit) {
                Ok(Self::Literal(input.parse()?))
            } else {
                match input.parse::<Expr>() {
                    Ok(expr) => Ok(CfgValue::Expr(expr)),
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
