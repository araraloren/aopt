use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::DeriveInput;
use syn::Generics;

use crate::config::Configs;
use crate::config::FetchKind;
use crate::error;

use super::GenericsModifier;

#[derive(Debug)]
pub struct FetchGenerator<'a> {
    ident: &'a Ident,

    configs: Configs<FetchKind>,

    generics: &'a Generics,
}

impl<'a> FetchGenerator<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let configs = Configs::<FetchKind>::parse_attrs("fetch", &input.attrs);

        Ok(Self {
            ident,
            configs,
            generics,
        })
    }

    pub fn gen_impl_for_struct(&self) -> syn::Result<TokenStream> {
        let ident = self.ident;
        let span = ident.span();
        let generics = self.generics.clone();
        let mut used_generics = Self::find_generics_t(&generics)?;
        let (_, type_generics, _) = generics.split_for_impl();
        let mut fetch_generics = GenericsModifier::new(generics.clone());
        let (impl_fetch, where_fetch);

        let handle = if self.only_have_cfgs(&[FetchKind::Inner, FetchKind::Map]) {
            let inner = self.configs.find_value(FetchKind::Inner);
            let map = self.configs.find_value(FetchKind::Map);

            if inner.is_none() {
                return Err(error(
                    span,
                    "`fetch` attribute: configuration `inner` can only using pair with `map`",
                ));
            }
            let inner = inner.unwrap();
            let handle = if let Some(map) = &map {
                quote! {
                    cote::prelude::fetch_uid_impl::<#inner, Set>(uid, set).map(#map)
                }
            } else {
                quote! {
                    cote::prelude::fetch_uid_impl::<#inner, Set>(uid, set)
                }
            };
            let inner_ident = Ident::new(&inner.to_token_stream().to_string(), span);

            used_generics.push(&inner_ident);
            (impl_fetch, where_fetch) = {
                let ret = fetch_generics.split_for_impl_fetch(&used_generics);

                (ret.0, ret.2)
            };
            handle
        } else {
            let handle = if let Some(handle) = self.configs.find_value(FetchKind::Handle) {
                quote! {
                    #handle(uid, set)
                }
            } else {
                quote! {
                    cote::prelude::fetch_uid_impl(uid, set).map(<#ident as cote::prelude::Infer>::infer_map)
                }
            };

            (impl_fetch, where_fetch) = {
                let ret = fetch_generics.split_for_impl_fetch(&used_generics);

                (ret.0, ret.2)
            };
            handle
        };

        Ok(quote! {
            impl #impl_fetch cote::prelude::Fetch<Set> for #ident #type_generics #where_fetch {
                fn fetch_uid(uid: cote::prelude::Uid, set: &mut Set) -> cote::Result<Self> {
                    #handle
                }
            }
        })
    }

    pub fn only_have_cfgs(&self, kinds: &[FetchKind]) -> bool {
        !self.configs.is_empty() && self.configs.iter().all(|v| kinds.contains(v.kind()))
    }

    pub fn find_generics_t(_self: &Generics) -> syn::Result<Vec<&Ident>> {
        let mut ret = vec![];

        for param in _self.params.iter() {
            if let syn::GenericParam::Type(ty_param) = param {
                let ident = &ty_param.ident;

                ret.push(ident);
            }
        }

        Ok(ret)
    }
}
