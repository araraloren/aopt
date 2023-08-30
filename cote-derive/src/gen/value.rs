use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, DeriveInput, Variant};

use crate::config::{Configs, ValueKind};
use crate::error;

#[derive(Debug)]
pub struct ValueGenerator<'a> {
    ident: &'a Ident,

    variants: Vec<&'a Variant>,

    variants_configs: Vec<Configs<ValueKind>>,

    configs: Configs<ValueKind>,
}

impl<'a> ValueGenerator<'a> {
    pub fn new(
        input: &'a DeriveInput,
        variants: Option<&'a Punctuated<Variant, Comma>>,
    ) -> syn::Result<Self> {
        let ident = &input.ident;
        let configs = Configs::<ValueKind>::parse_attrs("coteval", &input.attrs);
        let (variants, variants_configs) = if let Some(variants) = variants {
            let variants: Vec<&Variant> = variants.iter().collect();
            let configs = variants
                .iter()
                .map(|v| Configs::<ValueKind>::parse_attrs("coteval", &v.attrs))
                .collect();

            (variants, configs)
        } else {
            (vec![], vec![])
        };

        Ok(Self {
            ident,
            configs,
            variants,
            variants_configs,
        })
    }

    pub fn gen_impl(&self) -> syn::Result<TokenStream> {
        let ident = self.ident;
        let forward_cfg = self.configs.find_cfg(ValueKind::Forward);
        let map_cfg = self.configs.find_cfg(ValueKind::Map);
        let map_raw_cfg = self.configs.find_cfg(ValueKind::MapRaw);
        let map_str_cfg = self.configs.find_cfg(ValueKind::MapStr);
        let ignore_case = self.configs.has_cfg(ValueKind::IgCase);
        let impl_code = if let (Some(forward_cfg), Some(map_cfg)) = (forward_cfg, map_cfg) {
            if map_raw_cfg.is_some() || map_str_cfg.is_some() || ignore_case {
                return error(
                    ident.span(),
                    "`CoteVal` error: `forward` can only using pair with `map`".to_owned(),
                );
            }
            let forward = forward_cfg.value();
            let map = map_cfg.value();

            quote! {
                <#forward as cote::RawValParser>::parse(raw, ctx).map(#map)
            }
        } else {
            if map_raw_cfg.is_some() && map_str_cfg.is_some() {
                return error(
                    ident.span(),
                    "`CoteVal` error: `mapraw` or `mapstr` can not using on same type".to_owned(),
                );
            } else if map_cfg.is_some() {
                return error(
                    ident.span(),
                    "`CoteVal` error: `mapraw` or `mapstr` can not using with `map`".to_owned(),
                );
            } else if map_raw_cfg.is_some() && ignore_case {
                return error(
                    ident.span(),
                    "`CoteVal` error: `mapraw` can not using with `igcase`".to_owned(),
                );
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

            if let Some(cfg) = map_raw_cfg {
                let value = cfg.value();

                quote! {
                    #value(raw, ctx)
                }
            } else if let Some(cfg) = map_str_cfg {
                let value = cfg.value();

                quote! {
                    let name = #str_convert;
                    #value(name)
                }
            } else {
                if self.variants.is_empty() {
                    return  error(
                        ident.span(),
                        "`CoteVal` error: only can generate parsing code for enum type currently, conside using `forward` and `map` on struct".to_owned()
                    );
                }
                let mut mat_branchs = vec![];
                let enum_type = ident.to_string();

                for (variant, configs) in self.variants.iter().zip(self.variants_configs.iter()) {
                    let variant_ident = &variant.ident;
                    let variant_name = if ignore_case {
                        variant_ident.to_string().to_lowercase()
                    } else {
                        variant_ident.to_string()
                    };

                    if let Some(name) = configs.find_cfg(ValueKind::Name) {
                        let name = name.value();

                        mat_branchs.push(quote! {
                            #name => Ok(#ident::#variant_ident),
                        });
                    } else {
                        mat_branchs.push(quote! {
                            #variant_name => Ok(#ident::#variant_ident),
                        });
                    }
                    configs.iter().for_each(|v| {
                        if v.kind() == &ValueKind::Alias {
                            let alias = v.value();

                            mat_branchs.push(quote! {
                                #alias => Ok(#ident::#variant_ident),
                            });
                        }
                    })
                }
                mat_branchs.push(quote! {
                    _ => Err(cote::raise_failure!("Unknow value for enum type `{}`: {}", #enum_type, name).with_uid(uid)),
                });
                let mut match_code = quote! {};

                match_code.extend(mat_branchs);
                quote! {
                    let name = #str_convert;
                    let name = name.as_ref();
                    let uid = ctx.uid()?;

                    match name {
                        #match_code
                    }
                }
            }
        };

        Ok(quote! {
            impl cote::RawValParser for #ident {
                type Error = cote::aopt::Error;

                fn parse(raw: Option<&cote::RawVal>, ctx: &cote::Ctx) -> Result<Self, Self::Error> {
                    #impl_code
                }
            }
        })
    }
}
