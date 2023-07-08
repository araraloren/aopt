use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::Generics;

use crate::config::AlterKind;
use crate::config::Configs;

#[derive(Debug)]
pub struct AlterGenerator<'a> {
    ident: &'a Ident,

    generics: &'a Generics,
}

impl<'a> AlterGenerator<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;

        // not support configurations
        Configs::<AlterKind>::parse_attrs("alter", &input.attrs);

        Ok(Self { ident, generics })
    }

    pub fn gen_impl_for_struct(&self) -> syn::Result<TokenStream> {
        let ident = self.ident;
        let (impl_, type_, where_) = self.generics.split_for_impl();

        Ok(quote! {
            impl #impl_ cote::Alter for #ident #type_ #where_ {
                fn alter(hint: Hint, has_force: bool, cfg: &mut impl cote::ConfigValue) {
                    match hint {
                        cote::Hint::Opt => {
                            cfg.set_action(cote::Action::Set);
                            if !has_force {
                                cfg.set_force(false);
                            }
                        }
                        cote::Hint::Vec => {
                            cfg.set_action(cote::Action::App);
                            if !has_force {
                                cfg.set_force(true);
                            }
                        }
                        cote::Hint::OptVec => {
                            cfg.set_action(cote::Action::App);
                            if !has_force {
                                cfg.set_force(false);
                            }
                        }
                        cote::Hint::Null => {
                            cfg.set_action(cote::Action::Set);
                            if !has_force {
                                cfg.set_force(true);
                            }
                        }
                    }
                }
            }
        })
    }
}
