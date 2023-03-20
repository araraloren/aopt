use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::DeriveInput;
use syn::GenericParam;
use syn::Generics;
use syn::Lifetime;
use syn::Token;
use syn::WherePredicate;

use crate::config::Configs;
use crate::config::CoteKind;
use crate::config::SubKind;

use super::gen_default_policy_ty;
use super::gen_help_display_call;
use super::gen_option_ident;
use super::gen_option_uid_ident;
use super::OptUpdate;
use super::Update;
use super::HELP_OPTION_HELP;
use super::HELP_OPTION_IDENT;
use super::HELP_OPTION_NAME;
use super::HELP_OPTION_Q;
use super::HELP_OPTION_SHORT;
use super::HELP_OPTION_UID;
use super::MAIN_OPTION_IDENT;
use super::MAIN_OPTION_UID;
use super::POLICY_FWD;
use super::POLICY_PRE;

#[derive(Debug)]
pub struct CoteGenerator<'a> {
    name: TokenStream,

    ident: &'a Ident,

    configs: Configs<CoteKind>,

    generics: &'a Generics,

    lifetimes: Vec<&'a Ident>,

    predicates: Option<&'a Punctuated<WherePredicate, Token![,]>>,
}

impl<'a> CoteGenerator<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let params = &generics.params;
        let predicates = generics.where_clause.as_ref().map(|v| &v.predicates);
        let configs = Configs::<CoteKind>::parse_attrs("cote", &input.attrs);
        let name = if let Some(cfg) = configs.find_cfg(CoteKind::Name) {
            let value = cfg.value();

            quote! {
                String::from(#value)
            }
        } else {
            quote! {
                String::from(env!("CARGO_PKG_NAME"))
            }
        };
        let mut lifetimes = vec![];

        for param in params {
            match param {
                GenericParam::Type(_) => {}
                GenericParam::Lifetime(lifetime) => {
                    lifetimes.push(&lifetime.lifetime.ident);
                }
                GenericParam::Const(const_param) => {
                    abort! {
                        input,
                        "Parsing struct failed: Cote not support const parameter `{:?}`",
                        const_param,
                    }
                }
            }
        }

        Ok(Self {
            name,
            ident,
            configs,
            generics,
            lifetimes,
            predicates,
        })
    }

    pub fn get_ident(&self) -> &Ident {
        &self.ident
    }

    pub fn has_lifetime_ident(&self, other: &Ident) -> bool {
        for ident in &self.lifetimes {
            if ident == &other {
                return true;
            }
        }
        false
    }

    pub fn gen_where_clause(&self, has_zlifetime: bool) -> TokenStream {
        let where_predicate = if has_zlifetime {
            self.gen_where_predicate_zlifetime()
        } else {
            self.gen_where_predicate()
        };

        quote! {
            where
            P::Ser: aopt::ser::ServicesValExt + 'zlifetime,
            P::Error: Into<aopt::Error>,
            P::Set: aopt::prelude::Set + aopt::set::SetValueFindExt + 'zlifetime,
            P::Inv<'zlifetime>: aopt::ctx::HandlerCollection<'zlifetime, P::Set, P::Ser>,
            P: aopt::prelude::Policy + aopt::prelude::APolicyExt<P> + Default + 'zlifetime,
            aopt::prelude::SetCfg<P::Set>: aopt::prelude::Config + aopt::prelude::ConfigValue + Default,
            #where_predicate
        }
    }

    pub fn gen_style_manager_enable(&self) -> Option<TokenStream> {
        let has_combine = self.configs.has_cfg(CoteKind::Combine);
        let has_embedded = self.configs.has_cfg(CoteKind::EmbeddedPlus);

        if has_combine && has_embedded {
            Some(quote! {
                parser.enable_combined();
                parser.enable_embedded_plus();
            })
        } else if has_combine {
            Some(quote! {
                parser.enable_combined();
            })
        } else if has_embedded {
            Some(quote! {
                parser.enable_embedded_plus();
            })
        } else {
            None
        }
    }

    pub fn gen_help_display(&self, sub_configs: Option<&Configs<SubKind>>) -> Option<TokenStream> {
        let help_handler = gen_help_display_call(&self.name, &self.configs, sub_configs);
        let gen_abort_help = self.configs.has_cfg(CoteKind::AbortHelp);
        let gen_help = self.configs.has_cfg(CoteKind::Help);

        if gen_abort_help && gen_help {
            Some(quote! {
                let has_help_set = parser.find_val::<bool>(#HELP_OPTION_NAME).unwrap_or(&false) == &true;
                if ret.is_err() || !ret.as_ref().unwrap().status() || has_help_set  {
                    #help_handler
                    if has_help_set {
                        std::process::exit(0)
                    }
                }
            })
        } else if gen_help {
            Some(quote! {
                if parser.find_val::<bool>(#HELP_OPTION_NAME).unwrap_or(&false) == &true  {
                    #help_handler
                    std::process::exit(0)
                }
            })
        } else if gen_abort_help {
            Some(quote! {
                if ret.is_err() || !ret.as_ref().unwrap().status() {
                    #help_handler
                }
            })
        } else {
            None
        }
    }

    pub fn gen_where_predicate_zlifetime(&self) -> Option<TokenStream> {
        let mut code = quote! {};
        let zlifetime = Lifetime::new("'zlifetime", self.ident.span());

        for lifetime in self.lifetimes.iter() {
            let lifetime = Lifetime::new(&format!("'{}", lifetime), lifetime.span());

            code.extend(quote! {
                #zlifetime: #lifetime,
            });
        }
        if let Some(where_predicates) = self.predicates {
            Some(quote! { #code #where_predicates })
        } else if !self.lifetimes.is_empty() {
            Some(quote! { #code })
        } else {
            None
        }
    }

    pub fn gen_where_predicate(&self) -> Option<TokenStream> {
        self.predicates
            .map(|where_predicates| quote! { #where_predicates })
    }

    pub fn gen_policy_type(&self, has_sub_command: bool) -> syn::Result<TokenStream> {
        let policy_ty = self.configs.find_cfg(CoteKind::Policy);

        Ok(if let Some(policy_ty) = policy_ty {
            let policy_name = policy_ty.value().to_token_stream().to_string();
            let policy = gen_default_policy_ty(&policy_name);

            if let Some(policy) = policy {
                policy
            } else {
                policy_ty.value().to_token_stream()
            }
        } else {
            if has_sub_command {
                gen_default_policy_ty(POLICY_PRE).unwrap()
            } else {
                gen_default_policy_ty(POLICY_FWD).unwrap()
            }
        })
    }

    pub fn gen_main_option_update(&self, idx: usize) -> Option<OptUpdate> {
        let ident = self.ident;
        self.configs.find_cfg(CoteKind::On).map(|cfg| {
            let value = cfg.value();
            let ident = gen_option_ident(idx, ident.span());
            let uid = gen_option_uid_ident(idx, ident.span());

            (
                Some(quote! {
                    let #ident = {
                        ctor.new_with({
                            let mut config = aopt::prelude::SetCfg<P::Set>::default();
                            config.set_name(#MAIN_OPTION_IDENT);
                            <aopt::opt::Main>::infer_fill_info(&mut config, true);
                            config
                        }).map_err(Into::into)?
                    };
                }),
                Some(quote! {
                    let #uid = set.insert(#ident);
                }),
                Some(quote! {
                    parser.entry(#uid)?.on(#value);
                }),
            )
        })
    }

    pub fn gen_help_option_update(&self, idx: usize) -> Option<OptUpdate> {
        let ident = self.ident;
        self.configs.find_cfg(CoteKind::Help).map(|_| {
            let ident = gen_option_ident(idx, ident.span());
            let uid = gen_option_uid_ident(idx, ident.span());

            (
                Some(quote! {
                    let #ident = {
                        ctor.new_with({
                            let mut config = aopt::prelude::SetCfg::<P::Set>::default();
                            config.set_name(#HELP_OPTION_NAME);
                            config.add_alias(#HELP_OPTION_SHORT);
                            config.add_alias(#HELP_OPTION_Q);
                            config.set_help(#HELP_OPTION_HELP);
                            <bool>::infer_fill_info(&mut config, true);
                            config
                        }).map_err(Into::into)?
                    };
                }),
                Some(quote! {
                    #[allow(unused)]
                    let #uid = set.insert(#ident);
                }),
                None,
            )
        })
    }
}
