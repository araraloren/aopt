use syn::Ident;

use super::Kind;
use crate::error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlterKind {}

impl Kind for AlterKind {
    fn parse(input: &mut syn::parse::ParseStream) -> syn::Result<(Self, bool)> {
        let ident: Ident = input.parse()?;
        let option = ident.to_string();

        Err(error(
            input.span(),
            format!(
                "alter not support any configuration currently: {}",
                option.as_str()
            ),
        ))
    }
}
