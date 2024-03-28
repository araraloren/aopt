use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Field, GenericArgument, Ident, PathArguments, Type};

use crate::{config::SubKind, error};

use super::{FieldCfg, OptUpdate, Utils, POLICY_FWD};

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
        let policy_ty = self.gen_sub_policy_ty()?;
        let uid_ident = self.uid_ident();
        // using for access sub parser
        let sub_index = syn::Index::from(self.sub_index());
        let pass_help_to = help_uid.map(|id| {
            let uid_literal = Utils::id2uid_literal(id);

            quote! {
                if let Ok(value) = set.opt(#uid_literal)?.val::<bool>() {
                    if *value {
                        // if help set, pass original value to sub parser
                        args.push(ser.sve_take_val::<cote::prelude::RawVal>()?);
                    }
                }
            }
        });

        Ok(Some(quote! {
            parser.entry(#uid_ident)?.on(
                move |set: &mut cote::prelude::Parser<'inv, Set, Ser>, ser: &mut Ser, args: cote::prelude::ctx::Args,
                                index: cote::prelude::ctx::Index| {
                    use std::ops::Deref;

                    let mut args: Vec<cote::prelude::RawVal> = args.deref().clone().into();
                    let cmd = args.remove(*index.deref());
                    let cmd = cmd.get_str();
                    let cmd = cmd.ok_or_else(|| cote::prelude::raise_error!("Can not convert `{:?}` to &str", cmd))?;

                    // process help pass
                    // if we are jump into current handler, then we need pass original help option
                    #pass_help_to

                    let args = cote::prelude::ARef::new(cote::prelude::Args::from(args));
                    let parser = set.parser_mut(#sub_index)?;
                    let mut policy = <#policy_ty>::default();

                    // setup running ctx
                    parser.set_rctx(ser.sve_take_val::<cote::prelude::RunningCtx>()?);
                    parser.rctx_mut()?.add_name(cmd.to_owned());

                    // apply policy settings
                    <#inner_ty>::apply_policy_settings(&mut policy);

                    // parsing
                    let ret = cote::prelude::PolicyParser::parse_policy(parser, args, &mut policy);
                    let mut rctx = parser.take_rctx()?;

                    // check if we need display help for sub parser
                    if !rctx.display_help() {
                        <#inner_ty>::sync_rctx(&mut rctx, &ret, parser.optset())?;
                        if rctx.display_help() {
                            rctx.set_help_context(<#inner_ty>::new_help_context());
                        }
                        else {
                            rctx.pop_name(); // pop current name if not need display help
                        }
                    }

                    // insert back to owned parser
                    ser.sve_insert(rctx);

                    let ret = ret?;
                    let okay = ret.status();

                    Ok(if okay {
                        ser.sve_val_mut::<cote::prelude::RunningCtx>()?.clear_failed_info();
                        <#inner_ty as cote::ExtractFromSetDerive::<Set>>::try_extract(parser.optset_mut()).ok()
                    }
                    else {
                        ser.sve_val_mut::<cote::prelude::RunningCtx>()?
                            .add_failed_info(cote::prelude::FailedInfo{ name: cmd.to_owned(), retval: ret });
                        None
                    })
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
                            $cfg_ident.#method(#args);
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
        codes.push(quote! { cote::prelude::Cmd::infer_fill_info(&mut #cfg_ident)?; });
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
                    #ident: cote::prelude::SetExt::opt(#uid_literal).map(|v|v.val()).ok(),
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

    pub fn gen_sub_policy_ty(&self) -> syn::Result<TokenStream> {
        let policy_cfg = self.config.find_cfg(SubKind::Policy);

        Ok(policy_cfg
            .map(|policy_cfg| {
                let policy_name = policy_cfg.value().to_token_stream().to_string();
                let policy_ty = policy_cfg.value();

                Utils::gen_policy_ty(&policy_name).unwrap_or_else(|| {
                    quote! { <#policy_ty>::<'inv, Set, Ser> }
                })
            })
            .unwrap_or_else(|| Utils::gen_policy_ty(POLICY_FWD).unwrap()))
    }
}
