mod config;
mod gen;
mod value;

use gen::AlterGenerator;
use gen::Analyzer;
use gen::FetchGenerator;
use gen::InferGenerator;
use gen::ValueGenerator;
use proc_macro_error::abort;
use quote::quote;
use syn::parse_macro_input;
use syn::DataEnum;
use syn::DeriveInput;

#[proc_macro_derive(Cote, attributes(cote, arg, pos, cmd, sub, infer, alter, fetch))]
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

#[proc_macro_derive(CoteOpt, attributes(infer, alter, fetch))]
#[proc_macro_error::proc_macro_error]
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

    let ts = generator(&input).unwrap_or_else(|e| {
        abort! {
            input,
            "Failed to generate `CoteOpt` code for type: {:?}",
            e
        }
    });

    quote! {
        #ts
    }
    .into()
}

#[proc_macro_derive(CoteVal, attributes(coteval))]
#[proc_macro_error::proc_macro_error]
pub fn parser_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let generator = |input, variants| ValueGenerator::new(input, variants)?.gen_impl();

    if let syn::Data::Enum(DataEnum { ref variants, .. }) = &input.data {
        let ts = generator(&input, Some(variants)).unwrap_or_else(|e| {
            abort! {
                input,
                "Failed to generate `CoteVal` code for enum: {:?}",
                e
            }
        });
        quote! {
            #ts
        }
        .into()
    } else {
        let ts = generator(&input, None).unwrap_or_else(|e| {
            abort! {
                input,
                "Failed to generate `CoteVal` code for enum: {:?}",
                e
            }
        });
        quote! {
            #ts
        }
        .into()
    }
}
