mod analyzer;
mod global;
mod value;

use proc_macro_error::abort;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(Cote, attributes(cote, arg, sub))]
#[proc_macro_error::proc_macro_error]
pub fn parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    analyzer::derive_parser(&input)
        .unwrap_or_else(|e| {
            abort! {
                input, "Cote macro generate failed: {:?}", e
            }
        })
        .into()
}
