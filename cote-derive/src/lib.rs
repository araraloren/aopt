mod config;
mod gen;
mod value;

use gen::Analyzer;
use proc_macro_error::abort;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(
    Cote,
    attributes(cote, arg, pos, cmd, sub, infer, alter, fetch, rawvalparser)
)]
#[proc_macro_error::proc_macro_error]
pub fn parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let analyzer = Analyzer::new(&input).unwrap_or_else(|e| {
        abort! {
            input,
            "Failed to analyzer `cote` configuration on struct: {:?}",
            e
        }
    });
    let impl_code = analyzer.gen_all().unwrap_or_else(|e| {
        abort! {
            input,
            "Failed to generate code for struct: {:?}",
            e
        }
    });

    quote! {
        #impl_code
    }
    .into()
}
