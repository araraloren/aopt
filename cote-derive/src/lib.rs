mod config;
mod gen;
mod value;

use gen::AlterGenerator;
use gen::Analyzer;
use gen::FetchGenerator;
use gen::InferGenerator;
use gen::ValueGenerator;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::DataEnum;
use syn::DeriveInput;

fn error<T>(spanned: impl Spanned, msg: String) -> syn::Result<T> {
    Err(syn::Error::new(spanned.span(), msg))
}

#[proc_macro_derive(Cote, attributes(cote, arg, pos, cmd, sub))]
pub fn parser(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let analyzer = Analyzer::new(&input).unwrap_or_else(|e| {
        error(
            input.span(),
            format!("Failed to analyzer `cote` configuration on struct: {:?}", e),
        )
        .unwrap()
    });
    let impl_code = analyzer
        .gen_all()
        .unwrap_or_else(syn::Error::into_compile_error);

    quote! {
        #impl_code
    }
    .into()
}

#[proc_macro_derive(CoteOpt, attributes(infer, alter, fetch))]
pub fn parser_opt(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let generator = |input| -> syn::Result<proc_macro2::TokenStream> {
        let mut ret = quote! {};
        let fg = FetchGenerator::new(input)?;
        let ig = InferGenerator::new(input)?;
        let ag = AlterGenerator::new(input)?;

        ret.extend(fg.gen_impl_for_struct()?);
        ret.extend(ig.gen_impl_for_struct()?);
        ret.extend(ag.gen_impl_for_struct()?);

        Ok(ret)
    };

    let ts = generator(&input).unwrap_or_else(syn::Error::into_compile_error);

    quote! {
        #ts
    }
    .into()
}

#[proc_macro_derive(CoteVal, attributes(coteval))]
pub fn parser_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let generator = |input, variants| ValueGenerator::new(input, variants)?.gen_impl();

    if let syn::Data::Enum(DataEnum { ref variants, .. }) = &input.data {
        let ts = generator(&input, Some(variants)).unwrap_or_else(syn::Error::into_compile_error);
        quote! {
            #ts
        }
        .into()
    } else {
        let ts = generator(&input, None).unwrap_or_else(syn::Error::into_compile_error);
        quote! {
            #ts
        }
        .into()
    }
}
