mod parser;
mod global;

#[derive(Debug, Clone)]
enum CfgValue {
    Literal(Lit),

    Expr(Expr),

    Call(Vec<Expr>),

    Null,
}

impl Default for CfgValue {
    fn default() -> Self {
        Self::Null
    }
}

use syn::{DeriveInput, parse_macro_input, Lit, Expr};

#[proc_macro_derive(Cote, attributes(cote))]
#[proc_macro_error::proc_macro_error]
pub fn parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    parser::derive_parser(input).into()
}
