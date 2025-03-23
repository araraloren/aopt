use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Field, GenericArgument, Ident, PathArguments, Type};

use crate::{config::SubKind, error};

use super::{FieldCfg, OptUpdate, Utils};

#[derive(Debug)]
pub struct SubGenerator<'a> {
    index: usize,

    name: TokenStream,

    ident: Ident,

    uid_ident: Ident,

    inner_ty: Type, // type without option, sub is always wrapped with Option

    config: FieldCfg<'a, SubKind>,
}

impl<'a> SubGenerator<'a> {
    pub fn new(field: &'a Field, id: u64, index: usize) -> syn::Result<Self> {
        let config = FieldCfg::new(id, field, super::AttrKind::Sub)?;
        let ident = Utils::id2opt_ident(id, field.span());
        let uid_ident = Utils::id2opt_uid_ident(id, field.span());
        let inner_ty = Self::gen_inner_ty(&field.ty)?;
        let name = config
            .find_value(SubKind::Name)
            .map(|v| v.to_token_stream())
            .unwrap_or_else(|| config.ident().to_string().to_token_stream());

        Ok(Self {
            index,
            name,
            config,
            ident,
            inner_ty,
            uid_ident,
        })
    }

    pub fn uid(&self) -> u64 {
        self.config.id()
    }

    pub fn ty(&self) -> &'a Type {
        self.config.ty()
    }

    pub fn orig_ident(&self) -> &'a Ident {
        self.config.ident()
    }

    pub fn sub_index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> &TokenStream {
        &self.name
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn uid_ident(&self) -> &Ident {
        &self.uid_ident
    }

    pub fn inner_ty(&self) -> &Type {
        &self.inner_ty
    }

    pub fn gen_opt_update(&self, help_uid: Option<u64>) -> syn::Result<OptUpdate> {
        let c = self.gen_opt_create()?;
        let i = self.gen_opt_insert()?;
        let h = self.gen_opt_handler(help_uid)?;

        Ok(OptUpdate {
            h,
            ..Default::default()
        }
        .with_create(c)
        .with_insert(i))
    }

    pub fn gen_opt_insert(&self) -> syn::Result<TokenStream> {
        let ident = self.ident();
        let uid_ident = self.uid_ident();
        let uid_literal = Utils::id2uid_literal(self.uid());

        Utils::gen_opt_insert(ident, uid_ident, &uid_literal)
    }

    pub fn gen_opt_handler(&self, help_uid: Option<u64>) -> syn::Result<Option<TokenStream>> {
        let inner_ty = self.inner_ty();
        let policy_new = self.gen_sub_policy_new()?;
        let uid_ident = self.uid_ident();
        // using for access sub parser
        let sub_index = syn::Index::from(self.sub_index());
        let pass_help_to = help_uid.map(|id| {
            let uid_literal = Utils::id2uid_literal(id);

            quote! {
                if let Ok(value) = cote::prelude::OptValueExt::val::<bool>(cote::prelude::SetExt::opt(set, #uid_literal)?) {
                    if *value {
                        let help_val = set.app_data::<std::ffi::OsString>()?.clone();
                        // if help set, pass original value to sub parser
                        args.push(help_val);
                    }
                }
            }
        });

        Ok(Some(quote! {
            parser.entry(#uid_ident)?.on(
                move |set: &mut cote::prelude::Parser<'inv, Set>, ctx: &mut cote::prelude::Ctx| {
                    let index = ctx.idx()?;
                    let mut args: Vec<_> = ctx.args().iter().map(|v|v.to_os_string()).collect();
                    let cmd = args.remove(index);
                    let cmd = cmd.to_str();
                    let cmd = cmd.ok_or_else(|| cote::prelude::raise_error!("can not convert `{:?}` to &str", cmd))?;

                    // process help pass
                    // if we are jump into current handler, then we need pass original help option
                    #pass_help_to

                    let args = cote::prelude::Args::from(args);
                    let mut policy = #policy_new;

                    // checking running ctx
                    let mut rctx = cote::prelude::AppStorage::app_data_mut::<cote::prelude::RunningCtx>(set.ctx_service())?;
                    let sub_level = rctx.sub_level() as usize;

                    // if other sub command successed, skip the sub command
                    let ret = if rctx.frame_mut(sub_level)
                        .map(|v|v.failure.is_none()) == Some(true) {
                        None
                    }
                    else {
                        // clone a running ctx, make a new frame
                        let frame_len = rctx.frames().len();
                        let mut rctx = rctx.reset_at(sub_level as u8);
                        let mut frame = cote::prelude::Frame::new(set.parser_mut(#sub_index)?.name().clone());

                        // incrment sub level and push frame to running ctx
                        rctx.inc_sub_level().push_frame(frame);

                        // set running ctx for sub parser
                        cote::prelude::AppStorage::set_app_data(set.parser_mut(#sub_index)?.ctx_service(), rctx);

                        // apply policy settings
                        <#inner_ty>::apply_policy_settings(&mut policy);

                        // transfer app data to sub parser
                        set.transfer_appser_to_subparser(#sub_index);

                        // parsing
                        let ret = cote::prelude::PolicyParser::parse_policy(set.parser_mut(#sub_index)?, args, &mut policy);

                        // transfer app data from sub parser
                        set.transfer_appser_from_subparser(#sub_index);

                        // take running ctx from sub parser
                        let mut rctx = cote::prelude::AppStorage::take_app_data::<cote::prelude::RunningCtx>(
                            set.parser_mut(#sub_index)?.ctx_service()
                        )?;

                        // decrement sub level
                        rctx.dec_sub_level();
                        // skip if the sub parser has already set the help flag
                        if !rctx.display_help() {
                            <#inner_ty>::sync_rctx(&mut rctx, &ret, set.parser_mut(#sub_index)?.optset(), true)?;
                            if rctx.display_help() {
                                rctx.set_help_context(<#inner_ty>::new_help_context());
                            }
                        }

                        let ret = ret?;
                        let okay = ret.status();

                        if okay {
                            // pass running ctx to other sub command
                            cote::prelude::AppStorage::set_app_data(set.ctx_service(), rctx);
                            <#inner_ty as cote::ExtractFromSetDerive::<Set>>::try_extract(
                                set.parser_mut(#sub_index)?.optset_mut()
                            ).ok()
                        }
                        else {
                            if rctx.frames().len() > frame_len {
                                if let Some(frame) = rctx.frame_mut(sub_level) {
                                    frame.failure = Some(cote::prelude::Failure::new(cmd.to_owned(), ret));
                                }
                                // replace the running ctx with current one
                                cote::prelude::AppStorage::set_app_data(set.ctx_service(), rctx);
                            }
                            None
                        }
                    };

                    Ok(ret)
                }
            );
        }))
    }

    pub fn gen_opt_create(&self) -> syn::Result<TokenStream> {
        let field_span = self.ident().span();
        let field_cfg = &self.config;
        let cfg_ident = Ident::new("cfg", field_span);
        let mut codes = vec![];

        codes.push(SubKind::Name.simple(&cfg_ident, &self.name)?);
        for cfg in self.config.configs().iter() {
            let cfg_value = cfg.value();
            let kind = cfg.kind();

            match kind {
                SubKind::Alias | SubKind::Hint | SubKind::Help | SubKind::Force => {
                    let value = cfg_value.to_token_stream();

                    codes.push(kind.simple(&cfg_ident, &value)?);
                }
                SubKind::MethodCall(method) => {
                    let method = Ident::new(method, field_span);
                    let value = cfg.value().clone();
                    let (_self, args) = value.split_call_args(field_span)?;
                    let caller = _self.to_token_stream().to_string();

                    codes.push(match caller.as_str() {
                        "config" | "cfg" => quote! {
                            #method::<Set>(&mut #cfg_ident, #args);
                        },
                        _ => quote! { #method(#cfg_value); },
                    });
                }
                _ => {}
            }
        }
        if let Some(help) = field_cfg
            .find_value(SubKind::Help)
            .map(|v| quote! { String::from(#v.trim()) })
            .or_else(|| field_cfg.collect_help_msgs())
        {
            codes.push(SubKind::Help.simple(&cfg_ident, &help)?);
        }
        codes.push(quote! { <cote::prelude::Cmd as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?; });
        Utils::gen_opt_create(self.ident(), Some(quote! { #(#codes)* }))
    }

    pub fn gen_try_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let is_refopt = self.config.find_cfg(SubKind::Ref).is_some();
        let is_mutopt = self.config.find_cfg(SubKind::Mut).is_some();
        let uid_literal = Utils::id2uid_literal(self.uid());
        let ident = self.orig_ident();

        if is_refopt && is_mutopt {
            Err(error(
                ident.span(),
                format!("Can not set both mut and ref on field `{}`", ident),
            ))
        } else if is_refopt {
            Ok((
                true,
                quote! {
                    #ident: cote::prelude::SetExt::opt(#uid_literal).map(cote::prelude::OptValueExt::val).ok(),
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: cote::prelude::fetch_uid_impl(#uid_literal, set).ok()
                },
            ))
        }
    }

    pub fn gen_inner_ty(ty: &Type) -> syn::Result<Type> {
        if let Type::Path(path) = ty {
            if let Some(segment) = path.path.segments.last() {
                let ident_str = segment.ident.to_string();

                if ident_str == "Option" {
                    if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                        if let Some(GenericArgument::Type(next_ty)) = ab.args.first().as_ref() {
                            return Ok(next_ty.clone());
                        }
                    }
                }
            }
        }
        Err(error(
            ty,
            "`sub` configuration only support `Option<T>`".to_owned(),
        ))
    }

    pub fn gen_sub_policy_new(&self) -> syn::Result<TokenStream> {
        let policy_cfg = self.config.find_cfg(SubKind::Policy);
        let inner_ty = self.inner_ty();

        Ok(policy_cfg
            .map(|policy_cfg| {
                let policy_name = policy_cfg.value().to_token_stream().to_string();
                let policy_ty = policy_cfg.value();

                Utils::gen_policy_ty(&policy_name)
                    .map(|ty| {
                        quote! { <#ty>::default() }
                    })
                    .unwrap_or_else(|| {
                        quote! { <<#policy_ty>::<'inv, Set>>::default() }
                    })
            })
            .unwrap_or_else(|| quote! { <#inner_ty>::into_policy_with::<'inv, Set>() }))
    }
}
