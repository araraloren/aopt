use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::Generics;

use crate::config::Configs;
use crate::config::InferKind;

#[derive(Debug)]
pub struct InferGenerator<'a> {
    ident: &'a Ident,

    configs: Configs<InferKind>,

    generics: &'a Generics,
}

impl<'a> InferGenerator<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let configs = Configs::<InferKind>::parse_attrs("infer", &input.attrs);

        Ok(Self {
            ident,
            configs,
            generics,
        })
    }

    pub fn gen_impl_for_struct(&self) -> syn::Result<TokenStream> {
        let ident = self.ident;
        let (impl_, type_, where_) = self.generics.split_for_impl();
        let mut codes = vec![];

        for config in self.configs.iter() {
            let value = config.value();

            codes.push(match config.kind() {
                InferKind::Val => {
                    quote! {
                        type Val = #value;
                    }
                }
                InferKind::Action => {
                    quote! {
                        fn infer_act() -> cote::Action {
                            #value
                        }
                    }
                }
                InferKind::Force => {
                    quote! {
                        fn infer_force() -> bool {
                            #value
                        }
                    }
                }
                InferKind::Ctor => {
                    quote! {
                        fn infer_ctor() -> cote::Str {
                            #value
                        }
                    }
                }
                InferKind::Index => quote! {
                    fn infer_index() -> Option<cote::Index> {
                        #value
                    }
                },
                InferKind::Style => quote! {
                    fn infer_style() -> Vec<cote::Style> {
                        #value
                    }
                },
                InferKind::IgName => quote! {
                    fn infer_ignore_name() -> bool {
                        #value
                    }
                },
                InferKind::IgAlias => quote! {
                    fn infer_ignore_alias() -> bool {
                        #value
                    }
                },
                InferKind::IgIndex => quote! {
                    fn infer_ignore_index() -> bool {
                        #value
                    }
                },
                InferKind::Valid => quote! {
                    fn infer_validator() -> Option<cote::ValValidator<Self::Val>> {
                        #value
                    }
                },
                InferKind::Init => quote! {
                    fn infer_initializer() -> Option<ValInitializer> {
                        #value
                    }
                },
                InferKind::Type => quote! {
                    fn infer_type_id() -> std::any::TypeId  {
                        #value
                    }
                },
                InferKind::Tweak => quote! {
                    fn infer_tweak_info<C>(cfg: &mut C)
                    where
                        Self: Sized + 'static,
                        Self::Val: cote::RawValParser,
                        C: cote::ConfigValue + Default,
                    {
                        #value(cfg)
                    }
                },
                InferKind::Fill => quote! {
                    fn infer_fill_info<C>(cfg: &mut C, ignore_infer: bool)
                    where
                        Self: Sized + 'static,
                        Self::Val: cote::RawValParser,
                        C: cote::ConfigValue + Default,
                    {
                        #value(cfg, ignore_infer)
                    }
                },
            });
        }
        if !self.configs.has_cfg(InferKind::Val) {
            codes.push(quote! {
                type Val = Self;
            })
        }
        let mut code = quote! {};

        code.extend(codes);
        Ok(quote! {
            impl #impl_ cote::Infer for #ident #type_ #where_ {
                #code
            }
        })
    }
}
