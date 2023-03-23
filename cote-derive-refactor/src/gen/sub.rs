use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Field;
use syn::Lit;
use syn::Type;

use crate::config::Configs;
use crate::config::SubKind;
use crate::gen::gen_elision_lifetime_ty;

use super::filter_comment_doc;
use super::gen_default_policy_ty;
use super::gen_option_ident;
use super::gen_option_uid_ident;
use super::gen_ty_without_option;
use super::CoteGenerator;
use super::OptUpdate;
use super::POLICY_FWD;

#[derive(Debug)]
pub struct SubGenerator<'a> {
    ty: &'a Type,

    name: TokenStream,

    ident: Option<&'a Ident>,

    docs: Vec<Lit>,

    configs: Configs<SubKind>,

    elision_lifetime_ty: Type,

    without_option_ty: Type,
}

impl<'a> SubGenerator<'a> {
    pub fn new(field: &'a Field, cote: &CoteGenerator<'a>) -> syn::Result<Self> {
        let ty = &field.ty;
        let ident = field.ident.as_ref();
        let attrs = &field.attrs;
        let docs = filter_comment_doc(attrs);
        let configs = Configs::parse_attrs("sub", attrs);
        let (_, elision_lifetime_ty) = gen_elision_lifetime_ty(cote, ty);
        let without_option_ty = gen_ty_without_option(&elision_lifetime_ty)?;
        let name = {
            if let Some(cfg) = configs.find_cfg(SubKind::Name) {
                cfg.value().to_token_stream()
            } else {
                ident
                    .unwrap_or_else(|| {
                        abort! {
                            ident,
                            "`arg` or `sub` not support empty field name"
                        }
                    })
                    .to_token_stream()
            }
        };

        Ok(Self {
            ty,
            name,
            ident,
            docs,
            configs,
            elision_lifetime_ty,
            without_option_ty,
        })
    }

    pub fn gen_field_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let is_refopt = self.configs.find_cfg(SubKind::Ref).is_some();
        let is_mutopt = self.configs.find_cfg(SubKind::Mut).is_some();
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
                    #ident: set.find_val(#name).ok(),
                },
            ))
        } else if is_mutopt {
            Ok((
                false,
                quote! {
                    #ident: set.take_val(#name).ok(),
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: set.take_val(#name).ok(),
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
            Some(self.gen_option_handler_insert(&uid)?),
        ))
    }

    pub fn gen_option_config_insert(&self, uid: &Ident, ident: &Ident) -> TokenStream {
        quote! {
            let #uid = set.insert(#ident);
        }
    }

    pub fn gen_option_config_new(&self, ident: &Ident) -> syn::Result<TokenStream> {
        let name = &self.name;
        let mut codes = vec![];
        let mut config = quote! {
            let mut config = aopt::prelude::SetCfg::<P::Set>::default();
            config.set_name(#name);
        };

        for cfg in self.configs.iter() {
            codes.push(match cfg.kind() {
                SubKind::Alias => {
                    let token = cfg.value().to_token_stream();

                    quote! {
                        config.add_alias(#token);
                    }
                }
                SubKind::Hint => {
                    let token = cfg.value().to_token_stream();

                    quote! {
                        config.set_hint(#token);
                    }
                }
                SubKind::Help => {
                    let token = cfg.value().to_token_stream();

                    quote! {
                        config.set_help(#token);
                    }
                }
                _ => {
                    quote! {}
                }
            })
        }
        if !self.configs.has_cfg(SubKind::Help) && !self.docs.is_empty() {
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
            codes.push(quote! {
                config.set_help({ #code message });
            })
        }
        codes.push(quote! {
            aopt::opt::Cmd::infer_fill_info(&mut config, true);
            config
        });
        config.extend(codes.into_iter());

        Ok(quote! {
            let #ident = {
                ctor.new_with({ #config }).map_err(Into::into)?
            };
        })
    }

    pub fn gen_option_handler_insert(&self, uid: &Ident) -> syn::Result<TokenStream> {
        let without_option_ty = &self.without_option_ty;
        let policy_ty = if let Some(policy_cfg) = self.configs.find_cfg(SubKind::Policy) {
            let policy_name = policy_cfg.value().to_token_stream().to_string();
            let policy = gen_default_policy_ty(&policy_name);

            policy.unwrap_or(policy_cfg.value().to_token_stream())
        } else {
            gen_default_policy_ty(POLICY_FWD).unwrap()
        };

        Ok(quote! {
            parser.entry(#uid)?.on(
                move |set: &mut P::Set, ser: &mut P::Ser, args: aopt::prelude::ctx::Args, index: aopt::prelude::ctx::Index| {
                    use std::ops::Deref;

                    let mut args = args.deref().clone().into_inner();
                    let pre_ser_names = ser.sve_val::<Vec<String>>()?;
                    let mut ser_names = pre_ser_names.clone();
                    let current_cmd = args.remove(*index.deref());
                    let current_cmd = current_cmd.get_str();

                    ser_names.push(current_cmd.ok_or_else(||
                        aopt::Error::raise_error(format!("can not convert `{:?}` to str", current_cmd)))?.to_owned()
                    );

                    let args = aopt::ARef::new(aopt::prelude::Args::from_vec(args));
                    let mut parser = <#without_option_ty as cote::IntoParserDerive<#policy_ty>>::into_parser()?;

                    parser.set_app_data(ser_names.clone())?;
                    parser.init()?;

                    let ret = parser.parse(args).map_err(Into::into);
                    let ret = ret?;
                    let ret_ctx = ret.ctx();
                    let ret_args = ret_ctx.args();
                    let ret_inner_ctx = ret_ctx.inner_ctx().ok();
                    let ret_e = ret.failure();

                    if ret.status() {
                        Ok(<#without_option_ty>::try_extract(parser.optset_mut()).ok())
                    }
                    else {
                        Err(aopt::Error::raise_error(
                            format!("Failed at command `{}` with `{}`: {}, inner_ctx = {}",
                            stringify!(#without_option_ty), ret_args, ret_e.display(),
                            if let Some(inner_ctx) = ret_inner_ctx {
                                format!("{}", inner_ctx)
                            } else {
                                format!("None")
                            }
                        )))
                    }
                }
            );
        })
    }
}
