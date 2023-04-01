use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Field;
use syn::Lit;
use syn::Type;

use crate::config::ArgKind;
use crate::config::Configs;

use super::check_in_path;
use super::filter_comment_doc;
use super::gen_option_ident;
use super::gen_option_uid_ident;
use super::OptUpdate;

#[derive(Debug)]
pub struct ArgGenerator<'a> {
    field_ty: &'a Type,

    name: TokenStream,

    ident: Option<&'a Ident>,

    docs: Vec<Lit>,

    configs: Configs<ArgKind>,
}

impl<'a> ArgGenerator<'a> {
    pub fn new(field: &'a Field) -> syn::Result<Self> {
        let field_ty = &field.ty;
        let ident = field.ident.as_ref();
        let attrs = &field.attrs;
        let docs = filter_comment_doc(attrs);
        let configs = Configs::parse_attrs("arg", attrs);
        let is_position = check_in_path(field_ty, "Pos")? || check_in_path(field_ty, "Cmd")?;
        let name = {
            if let Some(cfg) = configs.find_cfg(ArgKind::Name) {
                cfg.value().to_token_stream()
            } else {
                let ident = ident.unwrap_or_else(|| {
                    abort! {
                        ident,
                        "`arg` or `sub` not support empty field name"
                    }
                });
                let ident = ident.to_string();
                let name = if is_position {
                    ident
                } else if ident.chars().count() >= 2 {
                    format!("--{}", ident)
                } else {
                    format!("-{}", ident)
                };

                quote! { #name }
            }
        };

        Ok(Self {
            field_ty,
            name,
            ident,
            configs,
            docs,
        })
    }

    pub fn has_handler(&self) -> bool {
        self.configs.has_cfg(ArgKind::On)
            || self.configs.has_cfg(ArgKind::Then)
            || self.configs.has_cfg(ArgKind::Fallback)
    }

    pub fn gen_nodelay_for_delay_parser(&self) -> Option<TokenStream> {
        self.configs.find_cfg(ArgKind::NoDelay).map(|_| {
            let name = &self.name;

            quote! {
                parser.policy_mut().set_no_delay(#name);
            }
        })
    }

    pub fn gen_value_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let is_refopt = self.configs.find_cfg(ArgKind::Ref).is_some();
        let is_mutopt = self.configs.find_cfg(ArgKind::Mut).is_some();
        let ident = self.ident;
        let name = &self.name;

        if is_refopt && is_mutopt {
            abort! {
                ident,
                "can not set both mut and ref on arg"
            }
        } else if is_refopt {
            Ok((
                true,
                quote! {
                    #ident: aopt::prelude::InferValueRef::infer_fetch(#name, set)?,
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: aopt::prelude::InferValueMut::infer_fetch(#name, set)?,
                },
            ))
        }
    }

    pub fn gen_option_update(&self, idx: usize) -> syn::Result<OptUpdate> {
        let ident = gen_option_ident(idx, self.ident.span());
        let uid = gen_option_uid_ident(idx, self.ident.span());

        Ok((
            Some(self.gen_option_config_new(&ident)?),
            Some(self.gen_option_config_insert(&uid, &ident)),
            self.gen_option_handler_insert(&uid),
        ))
    }

    pub fn gen_option_handler_insert(&self, uid: &Ident) -> Option<TokenStream> {
        if let Some(cfg) = self.configs.find_cfg(ArgKind::On) {
            let value = cfg.value();

            Some(
                if let Some(then_cfg) = self.configs.find_cfg(ArgKind::Then) {
                    let then = then_cfg.value();

                    quote! {
                        parser.entry(#uid)?.on(#value).then(#then);
                    }
                } else {
                    quote! {
                        parser.entry(#uid)?.on(#value);
                    }
                },
            )
        } else if let Some(cfg) = self.configs.find_cfg(ArgKind::Fallback) {
            let value = cfg.value();

            Some(
                if let Some(fallback) = self.configs.find_cfg(ArgKind::Then) {
                    let then = fallback.value();

                    quote! {
                        parser.entry(#uid)?.fallback(#value).then(#then);
                    }
                } else {
                    quote! {
                        parser.entry(#uid)?.fallback(#value);
                    }
                },
            )
        } else {
            None
        }
    }

    pub fn gen_option_config_insert(&self, uid: &Ident, ident: &Ident) -> TokenStream {
        if self.has_handler() {
            quote! {
                let #uid = set.insert(#ident);
            }
        } else {
            quote! {
                set.insert(#ident);
            }
        }
    }

    pub fn gen_option_config_new(&self, ident: &Ident) -> syn::Result<TokenStream> {
        let ty = &self.field_ty;
        let name = &self.name;
        let mut codes = vec![];
        let mut value = None;
        let mut config = quote! {
            let mut config = aopt::prelude::SetCfg::<P::Set>::default();
            config.set_name(#name);
        };

        // generate option create
        for cfg in self.configs.iter() {
            codes.push(
                    match cfg.kind() {
                        ArgKind::Hint => {
                            let token = cfg.value().to_token_stream();

                            quote! {
                                config.set_hint(#token);
                            }
                        }
                        ArgKind::Value => {
                            let token = cfg.value().to_token_stream();

                            value = Some(token.clone());
                            quote! {
                                config.set_initializer(aopt::prelude::ValInitializer::new_value(<ValueType>::from(#token)));
                            }
                        }
                        ArgKind::Values => {
                            let token = cfg.value().to_token_stream();

                            value = Some(token.clone());
                            quote! {
                                let values = #token.into_iter().map(|v|<ValueType>::from(v)).collect::<Vec<ValueType>>();
                                config.set_initializer(aopt::prelude::ValInitializer::new_values(values));
                            }
                        }
                        ArgKind::Alias => {
                            let token = cfg.value().to_token_stream();

                            quote! {
                                config.add_alias(#token);
                            }
                        }
                        ArgKind::Index => {
                            let token = cfg.value().to_token_stream();

                            quote! {
                                config.set_index(aopt::prelude::Index::parse(#token)?);
                            }
                        }
                        ArgKind::Force => {
                            quote! {
                                config.set_force(true);
                            }
                        }
                        ArgKind::NoForce => {
                            quote! {
                                config.set_force(false);
                            }
                        }
                        ArgKind::Action => {
                            let token = cfg.value().to_token_stream();

                            quote! {
                                config.set_action(#token);
                            }
                        }
                        ArgKind::Validator => {
                            let token = cfg.value().to_token_stream();

                            quote! {
                                config.set_storer(aopt::prelude::ValStorer::new_validator::<#ty>(#token));
                            }
                        }
                        _ => {
                            quote!{ }
                        }
                    }
               )
        }
        let help = if let Some(cfg) = self.configs.find_cfg(ArgKind::Help) {
            let value = cfg.value();
            Some(quote! { let mut message = String::from(#value.trim()); })
        } else if !self.docs.is_empty() {
            Some({
                let mut code = quote! {
                    let mut message = String::default();
                };
                let mut iter = self.docs.iter();

                if let Some(doc) = iter.next() {
                    code.extend(quote! {
                        message.push_str(#doc.trim());
                    });
                }
                for doc in iter {
                    code.extend(quote! {
                        message.push_str(" ");
                        message.push_str(#doc.trim());
                    });
                }
                code
            })
        } else {
            None
        };
        if let Some(mut help) = help {
            if let Some(value) = &value {
                let value_string = value.to_string();

                help.extend(quote! {
                    message.push_str(" ");
                    message.push_str("[");
                    message.push_str(#value_string.trim());
                    message.push_str("]");
                });
            }
            codes.push(quote! {
                config.set_help({ #help message });
            })
        }
        codes.push(quote! {
            <#ty as aopt::prelude::Infer>::infer_fill_info(&mut config, true);
            config
        });
        if value.is_some() {
            config.extend(quote! {
                 type ValueType = <#ty as aopt::prelude::Infer>::Val;
            });
        }
        config.extend(codes.into_iter());

        Ok(quote! {
            let #ident = {
                ctor.new_with({ #config }).map_err(Into::into)?
            };
        })
    }
}
