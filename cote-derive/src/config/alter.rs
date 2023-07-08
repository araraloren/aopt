use proc_macro_error::abort;
use syn::Ident;

use super::Kind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlterKind {}

impl Kind for AlterKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        match option.as_str() {
            _ => {
                abort! {
                    input.span(),
                    "alter not support any configuration currently"
                }
            }
        }
    }
}
