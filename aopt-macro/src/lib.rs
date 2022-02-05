extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, Expr};

#[derive(Debug)]
struct GetoptArgs {
    iter: Expr,
    parsers: Punctuated<Expr, Comma>,
}

impl Parse for GetoptArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parsers = Punctuated::new();
        let iter: Expr = input.parse()?;
        let comma: Comma = input.parse()?;

        while !input.is_empty() {
            let parser: Expr = input.parse()?;
            let next_comma: syn::Result<Comma> = input.parse();

            parsers.push(parser);
            match next_comma {
                Ok(is_comma) => {
                    parsers.push_punct(is_comma);
                }
                Err(_) => {
                    // last parser
                    parsers.push_punct(comma);
                }
            }
        }

        Ok(Self { iter, parsers })
    }
}

#[proc_macro]
pub fn getoptd(input: TokenStream) -> TokenStream {
    let getopt_args = parse_macro_input!(input as GetoptArgs);
    let iterator = &getopt_args.iter;
    let iterator = quote! { #iterator };

    let mut getopt_init = quote! {
        let mut parsers = vec![];
    };

    getopt_init.extend(getopt_args.parsers.iter().map(|p| match p {
        Expr::Path(path) => {
            quote! {
                parsers.push(&mut #path);
            }
        }
        Expr::Reference(reference) => {
            if reference.mutability.is_some() {
                quote! {
                    #reference
                }
            } else {
                syn::Error::new_spanned(reference, "need an instance or a mutable reference")
                    .to_compile_error()
            }
        }
        expr => {
            quote! {
                parsers.push(#expr);
            }
        }
    }));

    let ret = quote! {{
        #getopt_init
        getopt_dynparser(#iterator, parsers)
    }};
    ret.into()
}

#[proc_macro]
pub fn getopt(input: TokenStream) -> TokenStream {
    let getopt_args = parse_macro_input!(input as GetoptArgs);
    let iterator = &getopt_args.iter;
    let iterator = quote! { #iterator };

    let mut getopt_init = quote! {
        let mut parsers = vec![];
    };

    getopt_init.extend(getopt_args.parsers.iter().map(|p| match p {
        Expr::Path(path) => {
            quote! {
                parsers.push(&mut #path);
            }
        }
        Expr::Reference(reference) => {
            if reference.mutability.is_some() {
                quote! {
                    #reference
                }
            } else {
                syn::Error::new_spanned(reference, "need an instance or a mutable reference")
                    .to_compile_error()
            }
        }
        expr => {
            quote! {
                parsers.push(#expr);
            }
        }
    }));

    let ret = quote! {{
        #getopt_init
        getopt_parser(#iterator, parsers)
    }};
    ret.into()
}
