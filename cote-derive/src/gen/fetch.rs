use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::DeriveInput;
use syn::GenericParam;
use syn::Generics;
use syn::Token;
use syn::WherePredicate;

use crate::config::Config;
use crate::config::Configs;
use crate::config::FetchKind;
use crate::error;

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
        let (params, where_clause) = self.get_generics_params();
        let (impl_, type_, where_) = self.split_impl_for_struct(params, where_clause);

        let (mut scalar, mut vector) = (
            quote! {
                set.take_val(name)
            },
            quote! {
                set.take_vals(name)
            },
        );

        if !self.configs.is_empty() {
            if self.only_have_cfgs(&[FetchKind::Inner, FetchKind::Map]) {
                let inner = self.find_cfg(FetchKind::Inner);
                let map = self.find_cfg(FetchKind::Map);

                if inner.is_none() || map.is_none() {
                    return error(
                        ident.span(),
                        "`fetch` attribute: configuration `inner` can only using pair with `map`"
                            .to_owned(),
                    );
                }
                let inner = inner.unwrap();
                let map = map.unwrap();
                let inner_cfg = inner.value();
                let map_cfg = map.value();

                scalar = quote! {
                    set.take_val::<#inner_cfg>(name).map(#map_cfg)
                };
                vector = quote! {
                    set.take_vals::<#inner_cfg>(name)
                       .map(
                            |vals| vals.into_iter().map(#map_cfg).collect()
                        )
                };
            } else {
                return error(
                    ident.span(),
                    "Configuration `inner` can only using pair with `map`".to_owned(),
                );
            }
            if let Some(cfg) = self.find_cfg(FetchKind::Scalar) {
                let scalar_cfg = cfg.value();

                scalar = quote! {
                    #scalar_cfg(name, set)
                };
            }
            if let Some(cfg) = self.find_cfg(FetchKind::Vector) {
                let vector_cfg = cfg.value();

                vector = quote! {
                    #vector_cfg(name, set)
                };
            }
        }
        Ok(quote! {
            impl #impl_ cote::Fetch<'set> for #ident #type_ #where_ {
                fn fetch<S: cote::SetValueFindExt>(name: impl cote::ConfigBuild<cote::SetCfg<S>>, set: &'set mut S) -> Result<Self, aopt::Error>
                where
                    Self: cote::ErasedTy + Sized,
                    cote::SetCfg<S>: cote::ConfigValue + Default,
                {
                    #scalar
                }

                fn fetch_vec<S: cote::SetValueFindExt>(name: impl cote::ConfigBuild<cote::SetCfg<S>>, set: &'set mut S) -> Result<Vec<Self>, aopt::Error>
                where
                    Self: cote::ErasedTy + Sized,
                    cote::SetCfg<S>: cote::ConfigValue + Default,
                {
                    #vector
                }
            }
        })
    }

    pub fn find_cfg(&self, kind: FetchKind) -> Option<&Config<FetchKind>> {
        self.configs.iter().find(|v| v.kind() == &kind)
    }

    pub fn only_have_cfgs(&self, kinds: &[FetchKind]) -> bool {
        !self.configs.is_empty() && self.configs.iter().all(|v| kinds.contains(v.kind()))
    }

    pub fn split_impl_for_struct(
        &self,
        params: &Punctuated<GenericParam, Token![,]>,
        where_predicate: Option<&Punctuated<WherePredicate, Token![,]>>,
    ) -> (TokenStream, TokenStream, TokenStream) {
        (
            if params.is_empty() {
                quote! {
                    <'set>
                }
            } else {
                quote! {
                    <'set, #params>
                }
            },
            if params.is_empty() {
                quote! {}
            } else {
                quote! {
                    <#params>
                }
            },
            quote! { where #where_predicate },
        )
    }

    pub fn get_generics_params(
        &self,
    ) -> (
        &Punctuated<GenericParam, Token![,]>,
        Option<&Punctuated<WherePredicate, Token![,]>>,
    ) {
        let params = &self.generics.params;
        let where_predicate = self.generics.where_clause.as_ref().map(|v| &v.predicates);

        (params, where_predicate)
    }
}
