use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Field;
use syn::Index;
use syn::Lifetime;
use syn::Lit;
use syn::Type;

use crate::config::Configs;
use crate::config::SubKind;

use super::filter_comment_doc;
use super::gen_default_policy_ty;
use super::gen_option_ident;
use super::gen_option_uid_ident;
use super::gen_subapp_without_option;
use super::gen_ty_without_option;
use super::OptUpdate;
use super::APP_POSTFIX;
use super::HELP_OPTION_NAME;
use super::POLICY_FWD;

#[derive(Debug)]
pub struct SubGenerator<'a> {
    sub_id: usize,

    #[allow(unused)]
    field_ty: &'a Type,

    name: TokenStream,

    ident: Option<&'a Ident>,

    docs: Vec<Lit>,

    configs: Configs<SubKind>,

    without_option_ty: Type,
}

impl<'a> SubGenerator<'a> {
    pub fn new(field: &'a Field, sub_id: usize) -> syn::Result<Self> {
        let field_ty = &field.ty;
        let ident = field.ident.as_ref();
        let attrs = &field.attrs;
        let docs = filter_comment_doc(attrs);
        let configs = Configs::parse_attrs("sub", attrs);
        let without_option_ty = gen_ty_without_option(field_ty)?;
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
                    .to_string()
                    .to_token_stream()
            }
        };

        Ok(Self {
            sub_id,
            field_ty,
            name,
            ident,
            docs,
            configs,
            without_option_ty,
        })
    }

    pub fn name(&self) -> &TokenStream {
        &self.name
    }

    pub fn get_sub_id(&self) -> usize {
        self.sub_id
    }

    pub fn get_without_option_type(&self) -> &Type {
        &self.without_option_ty
    }

    pub fn gen_policy_type(&self) -> syn::Result<TokenStream> {
        let policy_ty = self.configs.find_cfg(SubKind::Policy);

        Ok(if let Some(policy_ty) = policy_ty {
            let policy_name = policy_ty.value().to_token_stream().to_string();
            let policy = gen_default_policy_ty(&policy_name);

            if let Some(policy) = policy {
                policy
            } else {
                policy_ty.value().to_token_stream()
            }
        } else {
            gen_default_policy_ty(POLICY_FWD).unwrap()
        })
    }

    pub fn gen_app_type(
        &self,
        lifetime: Option<Lifetime>,
        policy_ty: &TokenStream,
    ) -> syn::Result<TokenStream> {
        let sub_struct_app_ty = self.gen_struct_app_type()?;

        if let Some(lifetime) = lifetime {
            Ok(quote! {
                #sub_struct_app_ty<#lifetime, #policy_ty>
            })
        } else {
            Ok(quote! {
                #sub_struct_app_ty<'_, #policy_ty>
            })
        }
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
        } else {
            Ok((
                false,
                quote! {
                    #ident: set.take_val(#name).ok(),
                },
            ))
        }
    }

    pub fn gen_option_update(
        &self,
        idx: usize,
        sub_parser_tuple_ty: &TokenStream,
        is_process_help: bool,
        help_uid: Option<&Ident>,
    ) -> syn::Result<OptUpdate> {
        let ident = gen_option_ident(idx, self.ident.span());
        let uid = gen_option_uid_ident(idx, self.ident.span());

        Ok((
            Some(self.gen_option_config_new(&ident)?),
            Some(self.gen_option_config_insert(&uid, &ident)),
            Some(self.gen_option_handler_insert(
                &uid,
                sub_parser_tuple_ty,
                is_process_help,
                help_uid,
            )?),
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
            let mut config = aopt::prelude::SetCfg::<Set>::default();
            config.set_name(#name);
        };

        for cfg in self.configs.iter() {
            codes.push(match cfg.kind() {
                SubKind::Alias => {
                    let token = cfg.value();

                    quote! {
                        config.add_alias(#token);
                    }
                }
                SubKind::Hint => {
                    let token = cfg.value();

                    quote! {
                        config.set_hint(#token);
                    }
                }
                SubKind::Help => {
                    let token = cfg.value();

                    quote! {
                        config.set_help(#token);
                    }
                }
                SubKind::Force => {
                    let token = cfg.value();

                    quote! {
                        config.set_force(#token);
                    }
                }
                SubKind::RawCall(method) => {
                    let method = Ident::new(&method, ident.span());
                    let args = cfg.value();

                    quote! {
                        config.#method(#args);
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

    pub fn gen_option_handler_insert(
        &self,
        uid: &Ident,
        sub_parser_tuple_ty: &TokenStream,
        is_process_help: bool,
        help_uid: Option<&Ident>,
    ) -> syn::Result<TokenStream> {
        let without_option_ty = &self.without_option_ty;
        let policy_ty = self.gen_policy_type()?;
        let sub_id = self.get_sub_id();
        let sub_id = Index::from(sub_id);
        let pass_help_to_next = if is_process_help {
            let help_uid = help_uid.unwrap_or_else(|| {
                abort! {
                    uid,
                    "Failed generate help handler, found None of help uid"
                }
            });
            quote! {
                if let Ok(value) = set.opt(#help_uid)?.val::<bool>() {
                    if *value {
                        // pass a fake flag to next sub command
                        args.push(aopt::RawVal::from(#HELP_OPTION_NAME));
                    }
                }
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            parser.entry(#uid)?.on(
                move |set: &mut Set, ser: &mut Ser, args: aopt::prelude::ctx::Args, index: aopt::prelude::ctx::Index| {
                    use std::ops::Deref;

                    let mut args = args.deref().clone().into_inner();
                    let mut next_ctx = cote::RunningCtx::default();
                    let current_cmd = args.remove(*index.deref());
                    let current_cmd = current_cmd.get_str();
                    let current_cmd = current_cmd.ok_or_else(||
                        aopt::Error::raise_error(format!("can not convert `{:?}` to str", current_cmd)))?;

                    next_ctx.add_name(current_cmd.to_owned());
                    #pass_help_to_next

                    let args = aopt::ARef::new(aopt::prelude::Args::from_vec(args));
                    let sub_parser = ser.sub_parser_mut::<cote::CoteParser<Set, Inv, Ser>>(#sub_id)?;
                    let mut policy = <#without_option_ty>::gen_policy_with::<#policy_ty>();
                    let mut helper = <#without_option_ty>::gen_parser_helper::<Set, Inv, Ser>();

                    helper.set_inner_parser(&sub_parser);
                    helper.set_rctx(next_ctx)?;
                    let ret = helper.parse_with(&mut policy).map_err(Into::into);

                    helper.sync_rctx(&ret, true)?;
                    let mut sub_ctx = helper.take_rctx()?;

                    ser.sve_val_mut::<cote::RunningCtx>()?.sync_ctx(&mut sub_ctx);
                    let ret = ret?;

                    if ret.status() {
                        let sub_parser = ser.sub_parser_mut::<cote::CoteParser<Set, Inv, Ser>>(#sub_id)?;

                        ser.sve_val_mut::<cote::RunningCtx>()?.clear_failed_info();
                        Ok(<#without_option_ty>::try_extract(sub_parser.optset_mut()).ok())
                    }
                    else {
                        ser.sve_val_mut::<cote::RunningCtx>()?.sync_failed_info(&mut sub_ctx);
                        ser.sve_val_mut::<cote::RunningCtx>()?.add_failed_info((current_cmd.to_owned(), ret));
                        Ok(None)
                    }
                }
            );
        })
    }

    pub fn gen_struct_app_type(&self) -> syn::Result<Ident> {
        let ident = gen_subapp_without_option(&self.without_option_ty)?;

        Ok(Ident::new(
            &format!("{}{}", ident, APP_POSTFIX),
            ident.span(),
        ))
    }

    pub fn gen_update_help_context(&self) -> syn::Result<TokenStream> {
        let mut ret = quote! {};

        if let Some(head_cfg) = self.configs.find_cfg(SubKind::Head) {
            let value = head_cfg.value();

            ret.extend(quote! {
                context = context.with_head(String::from(#value));
            })
        }
        if let Some(foot_cfg) = self.configs.find_cfg(SubKind::Foot) {
            let value = foot_cfg.value();

            ret.extend(quote! {
                context = context.with_foot(String::from(#value));
            })
        }
        ret.extend(quote! { context });
        Ok(ret)
    }
}
