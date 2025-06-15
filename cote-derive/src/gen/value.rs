use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, DeriveInput, Variant};

use crate::config::{Configs, ValueKind};
use crate::error;

#[derive(Debug)]
pub struct ValueGenerator<'a> {
    ident: &'a Ident,

    variants: Vec<&'a Variant>,

    var_configs: Vec<Configs<ValueKind>>,

    configs: Configs<ValueKind>,
}

impl<'a> ValueGenerator<'a> {
    pub fn new(
        input: &'a DeriveInput,
        variants: Option<&'a Punctuated<Variant, Comma>>,
    ) -> syn::Result<Self> {
        let ident = &input.ident;
        let configs = Configs::<ValueKind>::parse_attrs("coteval", &input.attrs);
        let (variants, var_configs) = if let Some(variants) = variants {
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
            var_configs,
        })
    }

    pub fn gen_impl_for_enum(&self) -> syn::Result<TokenStream> {
        let ident = self.ident;
        let span = ident.span();
        let forward_cfg = self.configs.find_value(ValueKind::Forward);
        let map_cfg = self.configs.find_value(ValueKind::Map);
        let map_raw_cfg = self.configs.find_value(ValueKind::MapRaw);
        let map_str_cfg = self.configs.find_value(ValueKind::MapStr);
        let igcase = self.configs.has_cfg(ValueKind::IgCase);
        let impl_code = if let (Some(forward), Some(map)) = (forward_cfg, map_cfg) {
            if map_raw_cfg.is_some() || map_str_cfg.is_some() || igcase {
                return Err(error(
                    span,
                    "`CoteVal` error: `forward` can only using pair with `map`",
                ));
            }
            // map value using forward to other function
            quote! {
                <#forward as cote::prelude::RawValParser>::parse(raw, ctx).map(#map)
            }
        } else {
            Self::check_config(
                span,
                map_raw_cfg.is_some(),
                map_str_cfg.is_some(),
                map_cfg.is_some(),
                igcase,
            )?;
            let str_convert = if igcase {
                quote! { cote::prelude::raw2str(raw)?.to_lowercase() }
            } else {
                quote! { cote::prelude::raw2str(raw)? }
            };
            let ty_name = ident.to_string();

            if let Some(value) = map_raw_cfg {
                // map raw value to Self
                quote! { #value(raw, ctx) }
            } else if let Some(value) = map_str_cfg {
                // map str value to Self
                quote! { #value( #str_convert ) }
            } else {
                let mut branches = vec![];

                for (var, config) in self.variants.iter().zip(self.var_configs.iter()) {
                    let var_ident = &var.ident;
                    let var_name = if igcase {
                        var_ident.to_string().to_lowercase()
                    } else {
                        var_ident.to_string()
                    };
                    let name_cfg = config
                        .find_value(ValueKind::Name)
                        .map(|v| v.to_token_stream())
                        .unwrap_or(var_name.to_token_stream());
                    let alias_cfg = config.find_values(ValueKind::Alias);

                    branches.push(quote! {
                        #name_cfg #(| #alias_cfg)* => Ok(#ident::#var_ident),
                    })
                }

                quote! {
                    let name = #str_convert;
                    let uid = ctx.uid()?;

                    match name.as_ref() {
                        #(#branches)*

                        _ => Err(cote::prelude::failure!("Unknow value for enum type `{}`: {}", #ty_name, name).with_uid(uid)),
                    }
                }
            }
        };

        Ok(quote! {
            impl cote::prelude::RawValParser for #ident {
                type Error = cote::Error;

                fn parse(raw: Option<&std::ffi::OsStr>, ctx: &cote::prelude::Ctx) -> Result<Self, Self::Error> {
                    #impl_code
                }
            }
        })
    }

    pub fn check_config(
        span: Span,
        raw: bool,
        str: bool,
        map: bool,
        igcase: bool,
    ) -> syn::Result<()> {
        if raw && str {
            Err(error(
                span,
                "`CoteVal` error: `mapraw` or `mapstr` can not using on same type",
            ))
        } else if map {
            Err(error(
                span,
                "`CoteVal` error: `mapraw` or `mapstr` can not using with `map`",
            ))
        } else if raw && igcase {
            Err(error(
                span,
                "`CoteVal` error: `mapraw` can not using with `igcase`",
            ))
        } else {
            Ok(())
        }
    }
}
