use quote::quote;
use syn::DeriveInput;
use proc_macro2::TokenStream;

pub fn derive_parser(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;

    dbg!(&input.attrs);

    quote! {

    }
}