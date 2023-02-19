use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Field, Type, AngleBracketedGenericArguments, PathArguments, GenericArgument};

use crate::global::{CfgKind, Configurations, FieldCfg, GlobalCfg};

pub(crate) struct CodeGenerator<'a> {
    pub ident: &'a Ident,

    pub global_cfg: Configurations<GlobalCfg>,

    pub fields: Vec<(&'a Field, Configurations<FieldCfg>, Type)>,
}

impl<'a> CodeGenerator<'a> {
    // generate cote for current struct if has any sub parser
    pub fn using_cote(&self) -> bool {
        self.fields
            .iter()
            .any(|(_, cfg, _)| cfg.find_cfg(CfgKind::Policy).is_some())
    }

    pub fn generate(&self, lifetime: TokenStream) -> syn::Result<TokenStream> {
        if self.using_cote() {
            todo!()
        } else {
            self.generate_parser(lifetime)
        }
    }

    pub fn generate_parser(&self, lifetime: TokenStream) -> syn::Result<TokenStream> {
        let has_handler = self
            .fields
            .iter()
            .any(|(_, cfg, _)| cfg.find_cfg(CfgKind::On).is_some());
        let update = self.generate_update()?;
        let extract = self.generate_try_extract()?;
        let inner = quote! {
            fn update<'ylifetime>(parser: &'ylifetime mut Parser<'zlifetime, P>) -> Result<&'ylifetime mut Parser<'zlifetime, P>, Error> {
                #update
            }

            fn try_extract<'ylifetime>(parser: &'ylifetime mut Parser<'zlifetime, P>) -> Result<Self, Error> where Self: Sized {
                #extract
            }
        };
        let ident = self.ident;

        if has_handler {
            Ok(quote! {
                impl<#lifetime, 'zlifetime, P> CoteParserDeriveExt<'zlifetime, P> for #ident<#lifetime>
                where
                    P::Set: aopt::set::Set,
                    P::Error: Into<aopt::Error>,
                    P: aopt::parser::Policy + aopt::ext::APolicyExt<P> + Default,
                    SetCfg<P::Set>: aopt::opt::Config + aopt::opt::ConfigValue + Default,
                    P::Inv<'a>: aopt::ctx::HandlerCollection<'a, P::Set, P::Ser>,
                {
                    #inner
                }
            })
        } else {
            Ok(quote! {
                impl<#lifetime, 'zlifetime, P> CoteParserDeriveExt<'zlifetime, P> for #ident<#lifetime>
                where
                    P::Set: aopt::set::Set,
                    P::Error: Into<aopt::Error>,
                    P: aopt::parser::Policy + aopt::ext::APolicyExt<P> + Default,
                    SetCfg<P::Set>: aopt::opt::Config + aopt::opt::ConfigValue + Default,
                {
                    #inner
                }
            })
        }
    }

    pub fn generate_try_extract(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            todo!()
        })
    }

    pub fn generate_try_extract_field(field: &Field, config: &Configurations<FieldCfg>) -> syn::Result<TokenStream> {
        let ident = field.ident.as_ref().unwrap_or_else(|| {
            abort! {
                field,
                "missing field name"
            }
        });
        let name = format!("--{}", ident.to_string()).to_token_stream();
        let name = config.find_cfg(CfgKind::Name).map(|v|v.value.to_token_stream()).unwrap_or(name);
        let ty = &field.ty;

        Ok(quote! {
            match <#ty as aopt::value::Infer>::infer_convert() {
                aopt::value::InferConverter::Pop => {
                    set.take_val::<<#ty as aopt::value::Infer>::Val>(#name)?
                }
                aopt::value::InferConverter::PopOk => {
                    set.take_val::<<#ty as aopt::value::Infer>::Val>(#name).ok()
                }
                aopt::value::InferConverter::PopNew => {
                    <#ty>::from(set.take_val::<<#ty as aopt::value::Infer>::Val>(#name)?)
                }
                aopt::value::InferConverter::Val => {
                    set.find_val::<<#ty as aopt::value::Infer>::Val>(#name)?
                }
                aopt::value::InferConverter::ValOk => {
                    set.find_val::<<#ty as aopt::value::Infer>::Val>(#name).ok()
                }
                aopt::value::InferConverter::ValNew => {
                    <#ty>::from(set.find_val::<<#ty as aopt::value::Infer>::Val>(#name)?)
                }
                aopt::value::InferConverter::Val => {
                    set.find_val::<<#ty as aopt::value::Infer>::Val>(#name)?.as_ref()
                }
                aopt::value::InferConverter::Vals => {
                    set.find_vals::<<#ty as aopt::value::Infer>::Val>(#name)?
                }
                aopt::value::InferConverter::ValsAsRef => {
                    set.find_vals::<<#ty as aopt::value::Infer>::Val>(#name)?
                }
                aopt::value::InferConverter::Vals => {
                    set.find_vals::<<#ty as aopt::value::Infer>::Val>(#name)?
                }
                aopt::value::InferConverter::Vals => {
                    set.find_vals::<<#ty as aopt::value::Infer>::Val>(#name)?
                }
                _ => { }
            }
        })
    }

    pub fn generate_update(&self) -> syn::Result<TokenStream> {
        let mut update = quote! {
            let set = parser.optset_mut();
            let ctor_name = ctor_default_name();
            let ctor = set.ctor_mut(&ctor_name)?;


        };
        let mut create = vec![];
        let mut insert = vec![];
        let mut register = vec![];

        for (idx, field) in self.fields.iter().enumerate() {
            let ident = Ident::new(&format!("opt{}", idx), field.0.span());
            let create_option = Self::generate_create_option(field.0, &field.1, &field.2)?;
            let handler_cfg = field.1.find_cfg(CfgKind::On);

            create.push(quote! {
                let #ident = {
                    #create_option
                };
            });
            if let Some(cfg) = handler_cfg {
                let uid_ident = Ident::new(&format!("uid_opt{}", idx), field.0.span());
                let handler = cfg.value.to_token_stream();

                insert.push(quote! {
                    let #uid_ident = set.insert(#ident);
                });
                register.push(quote! {
                    parser.entry(#uid_ident).on(#handler)?;
                });
            } else {
                insert.push(quote! {
                    set.insert(#ident);
                });
            }
        }
        update.extend(create.into_iter());
        update.extend(insert.into_iter());
        update.extend(register.into_iter());
        update.extend(quote! { Ok(parser) });
        Ok(update)
    }

    pub fn generate_create_option(
        field: &Field,
        config: &Configurations<FieldCfg>,
        ty: &Type,
    ) -> syn::Result<TokenStream> {
        let config_register = Self::generate_field_config(field, config, ty)?;

        Ok(quote! {
            let config = { #config_register };
            ctor.new_with(config).map_err(Into::into)?
        })
    }

    pub fn generate_field_config(
        field: &Field,
        config: &Configurations<FieldCfg>,
        trimed_ty: &Type,
    ) -> syn::Result<TokenStream> {
        let ty = field.ty.clone();
        let mut ret = quote! {
            let mut config = SetCfg::<P::Set>::default();
        };
        let mut name_register = false;

        for cfg in config.cfgs.iter() {
            match cfg.kind {
                CfgKind::Policy => {}
                CfgKind::Hint => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        config.set_hint(#token);
                    });
                }
                CfgKind::Help => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        config.set_help(#token);
                    });
                }
                CfgKind::Name => {
                    let token = cfg.value.to_token_stream();

                    name_register = true;
                    ret.extend(quote! {
                        config.set_name(#token);
                    });
                }
                CfgKind::Value => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        config.set_initializer(aopt::value::ValInitializer::new_value(<<#ty as aopt::value::Infer>::Val>::from(#token)));
                    });
                }
                CfgKind::Values => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        let values = #token.into_iter().map(|v|<<#ty as aopt::value::Infer>::Val>::from(v)).collect::<Vec<<#ty as aopt::value::Infer>::Val>>();
                        config.set_initializer(aopt::value::ValInitializer::new_values(values));
                    });
                }
                CfgKind::Alias => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        config.add_alias(#token);
                    });
                }
                CfgKind::Action => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        config.set_action(#token);
                    });
                }
                CfgKind::Index => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        config.set_index(aopt::opt::Index::parse(#token)?);
                    });
                }
                CfgKind::Validator => {
                    let token = cfg.value.to_token_stream();

                    ret.extend(quote! {
                        config.set_storer(aopt::value::ValStorer::new_validator::<#ty>(#token));
                    });
                }
                CfgKind::On => {
                    // callback will register after option create
                }
                _ => {
                    abort! {
                        field, "Unsupport config kind on field: {:?}", cfg.kind
                    }
                }
            }
        }
        if name_register {
            ret.extend(quote! {
                <#trimed_ty>::infer_fill_info(&mut config, true);
                config
            });
        } else {
            let ident = field.ident.as_ref().unwrap_or_else(|| {
                abort! {
                    field,
                    "missing field name"
                }
            });
            let name = format!("--{}", ident.to_string());

            ret.extend(quote! {
                <#trimed_ty>::infer_fill_info(&mut config, true);
                if ! config.has_name() {
                    config.set_name(#name);
                }
                config
            });
        }

        Ok(ret)
    }
}
