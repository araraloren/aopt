mod global;
mod parser;
mod value;

use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Cote, attributes(cote))]
#[proc_macro_error::proc_macro_error]
pub fn parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    parser::derive_parser(input).into()
}
