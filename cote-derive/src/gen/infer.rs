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
        let (impl_inf, type_inf, where_inf) = self.generics.split_for_impl();
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
                        fn infer_act() -> cote::prelude::Action {
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
                        fn infer_ctor() -> String {
                            #value
                        }
                    }
                }
                InferKind::Index => quote! {
                    fn infer_index() -> Option<cote::prelude::Index> {
                        #value
                    }
                },
                InferKind::Style => quote! {
                    fn infer_style() -> Vec<cote::prelude::Style> {
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
                    fn infer_validator() -> Option<cote::prelude::ValValidator<Self::Val>> {
                        #value
                    }
                },
                InferKind::Init => quote! {
                    fn infer_initializer() -> Option<cote::prelude::ValInitializer> {
                        #value
                    }
                },
                InferKind::Type => quote! {
                    fn infer_type_id() -> std::any::TypeId  {
                        #value
                    }
                },
                InferKind::Map => quote! {
                    fn infer_map(val: Self::Val) -> Self {
                        (#value)(val)
                    }
                },
                InferKind::Mutable => quote! {
                    fn infer_mutable(&mut self, val: Self::Val)
                    where
                        Self: Sized,
                    {
                        (#value)(val)
                    }
                },
                InferKind::Tweak => quote! {
                    fn infer_tweak_info<C>(cfg: &mut C) -> cote::Result<()>
                    where
                        Self: Sized + 'static,
                        Self::Val: cote::prelude::RawValParser,
                        C: cote::prelude::ConfigValue + Default,
                    {
                        #value(cfg);
                        Ok(())
                    }
                },
                InferKind::Fill => quote! {
                    fn infer_fill_info<C>(cfg: &mut C) -> cote::Result<()>
                    where
                        Self: Sized + 'static,
                        Self::Val: cote::prelude::RawValParser,
                        C: cote::prelude::ConfigValue + Default,
                    {
                        #value(cfg, ignore_infer);
                        Ok(())
                    }
                },
            });
        }
        if !self.configs.has_cfg(InferKind::Val) {
            codes.push(quote! {
                type Val = Self;
            })
        }
        if !self.configs.has_cfg(InferKind::Map) {
            codes.push(quote! {
                fn infer_map(val: Self::Val) -> Self {
                    val
                }
            });
        }
        Ok(quote! {
            impl #impl_inf cote::prelude::Infer for #ident #type_inf #where_inf {
                #(#codes)*
            }
        })
    }
}
