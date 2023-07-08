use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, DeriveInput, Variant};

use crate::config::{Configs, ValueKind};

#[derive(Debug)]
pub struct ValueGenerator<'a> {
    ident: &'a Ident,

    variants: Vec<&'a Variant>,

    configs: Configs<ValueKind>,
}

impl<'a> ValueGenerator<'a> {
    pub fn new(
        input: &'a DeriveInput,
        variants: &'a Punctuated<Variant, Comma>,
    ) -> syn::Result<Self> {
        let ident = &input.ident;
        let configs = Configs::<ValueKind>::parse_attrs("rawvalparser", &input.attrs);
        let variants = variants.iter().map(|v| v).collect();

        Ok(Self {
            ident,
            configs,
            variants,
        })
    }

    pub fn gen_impl_for_enum(&self) -> syn::Result<TokenStream> {
        let ident = self.ident;
        let cfg_map = self.configs.has_cfg(ValueKind::Map);
        let cfg_mapstr = self.configs.has_cfg(ValueKind::MapStr);
        let ignore_case = self.configs.has_cfg(ValueKind::IgCase);

        if cfg_map && cfg_mapstr {
            abort! {
                ident,
                "Can not using configuration `map` and `mapstr` on same enum"
            }
        }
        let str_convert = if ignore_case {
            quote! {
                cote::raw2str(raw)?.to_lowercase();
            }
        } else {
            quote! {
                cote::raw2str(raw)?;
            }
        };

        let parsing = if cfg_map {
            let cfg = self.configs.find_cfg(ValueKind::Map).unwrap();
            let value = cfg.value();

            quote! {
                #value(raw, ctx)
            }
        } else if cfg_mapstr {
            let cfg = self.configs.find_cfg(ValueKind::MapStr).unwrap();
            let value = cfg.value();

            quote! {
                let name = #str_convert;
                #value(name)
            }
        } else {
            let mut matchs = vec![];
            let enum_type = ident.to_string();

            for variant in self.variants.iter() {
                let variant_ident = &variant.ident;
                let variant_name = if ignore_case {
                    variant_ident.to_string().to_lowercase()
                } else {
                    variant_ident.to_string()
                };

                matchs.push(quote! {
                    #variant_name => Ok(#ident::#variant_ident),
                });
            }
            matchs.push(quote! {
                _ => Err(cote::raise_failure!("Unknow value for enum type `{}`: {}", #enum_type, name).with_uid(uid)),
            });
            let mut match_code = quote! {};

            match_code.extend(matchs.into_iter());
            quote! {
                let name = #str_convert;
                let uid = ctx.uid()?;

                match name.as_str() {
                    #match_code
                }
            }
        };

        Ok(quote! {
            impl cote::RawValParser for #ident {
                type Error = cote::aopt::Error;

                fn parse(raw: Option<&cote::RawVal>, ctx: &cote::Ctx) -> Result<Self, Self::Error> {
                    #parsing
                }
            }
        })
    }
}
