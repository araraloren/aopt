use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, spanned::Spanned, Attribute, Field, GenericArgument, Generics, Ident,
    ImplGenerics, Lifetime, LifetimeParam, Lit, PathArguments, Type, TypeGenerics, TypeParam,
    WhereClause,
};

use crate::{
    config::{self, Config, Configs},
    error,
    value::Value,
};

pub const CONFIG_SUB: &str = "sub";
pub const CONFIG_ARG: &str = "arg";
pub const CONFIG_CMD: &str = "cmd";
pub const CONFIG_POS: &str = "pos";
pub const CONFIG_DOC: &str = "doc";
pub const POLICY_PRE: &str = "pre";
pub const POLICY_FWD: &str = "fwd";
pub const POLICY_DELAY: &str = "delay";
pub const HELP_OPTION: &str = "--help;-h=b: Display help message";

#[derive(Debug, Clone, Copy)]
pub enum AttrKind {
    Sub,

    Arg,

    Cmd,

    Pos,

    Main,
}

impl AttrKind {
    pub fn is_sub(&self) -> bool {
        matches!(self, AttrKind::Sub)
    }

    // pub fn is_arg(&self) -> bool {
    //     matches!(self, AttrKind::Arg)
    // }

    pub fn is_cmd(&self) -> bool {
        matches!(self, AttrKind::Cmd)
    }

    pub fn is_pos(&self) -> bool {
        matches!(self, AttrKind::Pos)
    }

    pub fn is_main(&self) -> bool {
        matches!(self, AttrKind::Main)
    }

    pub fn name(&self) -> &'static str {
        match self {
            AttrKind::Sub => CONFIG_SUB,
            AttrKind::Arg => CONFIG_ARG,
            AttrKind::Cmd => CONFIG_CMD,
            AttrKind::Pos => CONFIG_POS,
            AttrKind::Main => unreachable!("Main don't need this"),
        }
    }

    pub fn gen_infer(
        &self,
        ident: &Ident,
        cfg_ident: &Ident,
        hint: &WrapperTy,
    ) -> syn::Result<TokenStream> {
        match self {
            AttrKind::Cmd => {
                if !hint.is_null() {
                    Err(error(
                        cfg_ident.span(),
                        format!("Remove `Option` or `Vec` on `cmd` field `{}`", ident,),
                    ))
                } else {
                    let inner_ty = hint.inner_type();

                    Ok(quote! {
                        #cfg_ident.set_type::<#inner_ty>();
                        <cote::prelude::Cmd as cote::prelude::Alter>::alter(cote::prelude::Hint::Null, &mut #cfg_ident);
                        <cote::prelude::Cmd as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                    })
                }
            }
            AttrKind::Pos => {
                Ok(match hint {
                    WrapperTy::Opt(inner_ty) => {
                        quote! {
                            // using information of Pos<T>
                            #cfg_ident.set_type::<#inner_ty>();
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Alter>::alter(cote::prelude::Hint::Opt, &mut #cfg_ident);
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                        }
                    }
                    WrapperTy::Vec(inner_ty) => {
                        quote! {
                            // using information of Pos<T>
                            #cfg_ident.set_type::<#inner_ty>();
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Alter>::alter(cote::prelude::Hint::Vec, &mut #cfg_ident);
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                        }
                    }
                    WrapperTy::OptVec(inner_ty) => {
                        quote! {
                            // using information of Pos<T>
                            #cfg_ident.set_type::<#inner_ty>();
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Alter>::alter(cote::prelude::Hint::OptVec, &mut #cfg_ident);
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                        }
                    }
                    WrapperTy::Null(inner_ty) => {
                        quote! {
                            // using information of Pos<T>
                            #cfg_ident.set_type::<#inner_ty>();
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Alter>::alter(cote::prelude::Hint::Null, &mut #cfg_ident);
                            <cote::prelude::Pos<#inner_ty> as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                        }
                    }
                })
            }
            AttrKind::Arg => Ok(match hint {
                WrapperTy::Opt(inner_ty) => {
                    quote! {
                        <#inner_ty as cote::prelude::Alter>::alter(cote::prelude::Hint::Opt, &mut #cfg_ident);
                        <#inner_ty as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                    }
                }
                WrapperTy::Vec(inner_ty) => {
                    quote! {
                        <#inner_ty as cote::prelude::Alter>::alter(cote::prelude::Hint::Vec, &mut #cfg_ident);
                        <#inner_ty as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                    }
                }
                WrapperTy::OptVec(inner_ty) => {
                    quote! {
                        <#inner_ty as cote::prelude::Alter>::alter(cote::prelude::Hint::OptVec, &mut #cfg_ident);
                        <#inner_ty as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                    }
                }
                WrapperTy::Null(inner_ty) => {
                    quote! {
                        <#inner_ty as cote::prelude::Alter>::alter(cote::prelude::Hint::Null, &mut #cfg_ident);
                        <#inner_ty as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
                    }
                }
            }),
            _ => {
                unreachable!("In AttrKind, can not get here ...")
            }
        }
    }
}

#[derive(Debug)]
pub struct FieldCfg<'a, T> {
    id: u64,

    ty: &'a Type,

    kind: AttrKind,

    ident: &'a Ident,

    docs: Vec<Lit>,

    configs: Configs<T>,
}

impl<'a, T: config::Kind + PartialEq> FieldCfg<'a, T> {
    pub fn new(id: u64, field: &'a Field, kind: AttrKind) -> syn::Result<Self> {
        let ty = &field.ty;
        let ident = field.ident.as_ref();
        let ident = ident.ok_or_else(|| error(field.span(), "Not support unnamed field"))?;
        let configs = Configs::<T>::parse_attrs(kind.name(), &field.attrs);
        let docs = Self::filter_comment_doc(&field.attrs);

        Ok(Self {
            id,
            ty,
            kind,
            ident,
            configs,
            docs,
        })
    }

    pub fn filter_comment_doc(attrs: &[Attribute]) -> Vec<Lit> {
        let attrs = attrs.iter().filter(|v| v.path().is_ident(CONFIG_DOC));
        let mut ret = vec![];

        for attr in attrs {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = &meta.value {
                    ret.push(lit.clone());
                }
            }
        }
        ret
    }

    // With api, automate generated by api-gen ...
    // pub fn with_id(mut self, value: u64) -> Self {
    //     self.id = value;
    //     self
    // }

    // pub fn with_kind(mut self, value: AttrKind) -> Self {
    //     self.kind = value;
    //     self
    // }

    // pub fn with_docs(mut self, value: Vec<Lit>) -> Self {
    //     self.docs = value;
    //     self
    // }

    // pub fn with_configs(mut self, value: Configs<T>) -> Self {
    //     self.configs = value;
    //     self
    // }

    // Get api, automate generated by api-gen ...
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn kind(&self) -> AttrKind {
        self.kind
    }

    pub fn docs(&self) -> &[Lit] {
        &self.docs
    }

    pub fn configs(&self) -> &Configs<T> {
        &self.configs
    }

    pub fn ty(&self) -> &'a Type {
        self.ty
    }

    pub fn ident(&self) -> &'a Ident {
        self.ident
    }

    pub fn has_cfg(&self, kind: T) -> bool {
        self.configs.has_cfg(kind)
    }

    pub fn find_cfg(&self, kind: T) -> Option<&Config<T>> {
        self.configs.find_cfg(kind)
    }

    pub fn find_value(&self, kind: T) -> Option<&Value> {
        self.configs.find_cfg(kind).map(|v| v.value())
    }

    pub fn collect_help_msgs(&self) -> Option<TokenStream> {
        if self.docs().is_empty() {
            None
        } else {
            let docs = self.docs.iter();

            Some(quote! {
                [ #(#docs),* ].into_iter().map(|v|v.trim()).collect::<Vec<_>>().join(" ")
            })
        }
    }
}

#[derive(Debug)]
pub struct Utils;

impl Utils {
    pub fn ident2opt_name(ident: &str) -> String {
        if ident.chars().count() > 1 {
            format!("--{}", ident.replace('_', "-"))
        } else {
            format!("-{}", ident)
        }
    }

    pub fn id2opt_ident(id: u64, span: Span) -> Ident {
        Ident::new(&format!("option_{}", id), span)
    }

    pub fn id2opt_uid_ident(id: u64, span: Span) -> Ident {
        Ident::new(&format!("option_{}_uid", id), span)
    }

    pub fn id2uid_literal(id: u64) -> syn::Lit {
        syn::Lit::Verbatim(proc_macro2::Literal::u64_suffixed(id))
    }

    pub fn gen_opt_create(
        ident: &Ident,
        cfg_modifer: Option<TokenStream>,
    ) -> syn::Result<TokenStream> {
        Ok(quote! {
            let #ident = {
                let cfg = {
                    let mut cfg = cote::prelude::SetCfg::<Set>::default();

                    #cfg_modifer
                    cfg
                };
                set.ctor_mut(&ctor_name)?.new_with(cfg).map_err(Into::into)?
            };
        })
    }

    pub fn gen_opt_insert(
        ident: &Ident,
        uid_ident: &Ident,
        uid_literal: &syn::Lit,
    ) -> syn::Result<TokenStream> {
        Ok(quote! {
            let #uid_ident = set.insert(#ident);

            assert_eq!(#uid_ident, #uid_literal, "Oops! Uid must be equal here");
        })
    }

    pub fn gen_opt_handler<T>(
        uid_ident: &Ident,
        on: Option<&Config<T>>,
        fallback: Option<&Config<T>>,
        then: Option<&Config<T>>,
    ) -> syn::Result<Option<TokenStream>> {
        if on.is_some() && fallback.is_some() {
            Err(error(
                uid_ident.span(),
                "Can not set both `on` and `fallback` attribute at same time",
            ))
        } else {
            Ok(on
                .map(|handler| {
                    if let Some(then) = then {
                        quote! {
                            parser.entry(#uid_ident)?.on(#handler).then(#then);
                        }
                    } else {
                        quote! {
                            parser.entry(#uid_ident)?.on(#handler);
                        }
                    }
                })
                .or_else(|| {
                    fallback.map(|handler| {
                        if let Some(then) = then {
                            quote! {
                                parser.entry(#uid_ident)?.fallback(#handler).then(#then);
                            }
                        } else {
                            quote! {
                                parser.entry(#uid_ident)?.fallback(#handler);
                            }
                        }
                    })
                }))
        }
    }

    pub fn check_in_ty(ty: &Type, ty_name: &str) -> syn::Result<bool> {
        if let Type::Path(path) = ty {
            if let Some(segment) = path.path.segments.last() {
                let ident = segment.ident.to_string();

                if ident == ty_name {
                    return Ok(true);
                } else if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                    for arg in ab.args.iter() {
                        if let GenericArgument::Type(next_ty) = arg {
                            return Self::check_in_ty(next_ty, ty_name);
                        }
                    }
                }
            }
            Ok(false)
        } else {
            Err(error(ty, "Cote not support reference type"))
        }
    }

    pub fn gen_policy_ty(policy_name: &str) -> Option<TokenStream> {
        match policy_name {
            POLICY_PRE => Some(quote! {
                cote::prelude::PrePolicy<'inv, Set, Ser>
            }),
            POLICY_FWD => Some(quote! {
                cote::prelude::FwdPolicy<'inv, Set, Ser>
            }),
            POLICY_DELAY => Some(quote! {
                cote::prelude::DelayPolicy<'inv, Set, Ser>
            }),
            _ => None,
        }
    }

    pub fn gen_policy_default_ty(policy_name: &str) -> Option<TokenStream> {
        match policy_name {
            POLICY_PRE => Some(quote! {
                cote::prelude::PrePolicy<'inv, cote::prelude::ASet, cote::prelude::ASer>
            }),
            POLICY_FWD => Some(quote! {
                cote::prelude::FwdPolicy<'inv, cote::prelude::ASet, cote::prelude::ASer>
            }),
            POLICY_DELAY => Some(quote! {
                cote::prelude::DelayPolicy<'inv, cote::prelude::ASet, cote::prelude::ASer>
            }),
            _ => None,
        }
    }

    // variable name: `ret`, `rctx`, and `parser`
    pub fn gen_sync_ret(
        has_sub: bool,
        enable_abort: bool,
        enable_normal: bool,
        help_uid: Option<u64>,
    ) -> syn::Result<TokenStream> {
        let abort_help = enable_abort.then(|| {
            Some(quote! {
                if !ret.is_ok()
                    || !ret.as_ref().map(cote::prelude::Status::status).unwrap_or(true) {
                    rctx.set_display_help(true);
                    rctx.set_exit(false);
                }
            })
        });
        let normal_help = enable_normal.then(|| {
            let uid_literal = Utils::id2uid_literal(help_uid.unwrap());
            Some(quote! {
                if set.opt(#uid_literal)?.val::<bool>().ok() == Some(&true) {
                    rctx.set_display_help(true);
                    rctx.set_exit(true);
                    // if we have sub parsers and we not in sub parser
                    // running ctx not have sub parser flag
                    // then we should not exit to show the error of sub command
                    if #has_sub && !sub_parser && !rctx.sub_parser() {
                        rctx.set_exit(false);
                    }
                }
            })
        });

        Ok(quote! {
            #abort_help
            #normal_help
        })
    }
}

pub struct GenericsModifier(Generics);

impl GenericsModifier {
    pub fn new(generics: Generics) -> Self {
        Self(generics)
    }

    pub fn insert_lifetime(&mut self, lifetime: &str) -> &mut Self {
        self.0.params.insert(
            0,
            syn::GenericParam::from(LifetimeParam::new(Lifetime::new(lifetime, self.0.span()))),
        );
        self
    }

    pub fn append_type(&mut self, ty: &str) -> &mut Self {
        self.0
            .params
            .push(syn::GenericParam::from(TypeParam::from(Ident::new(
                ty,
                self.0.span(),
            ))));
        self
    }

    pub fn mod_for_ipd(&mut self, used: &[&Ident]) -> &mut Self {
        let orig_where = self.0.where_clause.as_ref().map(|v| &v.predicates);
        let alter = Self::gen_alter_for_ty(used);
        let fetch = Self::gen_fetch_for_ty(used, quote!('set), quote!(Set), true);
        let new_where: WhereClause = parse_quote! {
            where
            Set: cote::prelude::Set + cote::prelude::OptParser + cote::prelude::OptValidator + cote::prelude::SetValueFindExt + Default + 'inv,
            Ser: cote::prelude::ServicesValExt + Default + 'inv,
            cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default,
            <Set as cote::prelude::OptParser>::Output: cote::prelude::Information,
            #(#used: cote::prelude::Infer + cote::prelude::ErasedTy,)*
            #(<#used as cote::prelude::Infer>::Val: cote::prelude::RawValParser,)*
            #alter
            #fetch
            #orig_where
        };

        self.0.where_clause = Some(new_where);
        self.insert_lifetime("'inv");
        self.append_type("Set");
        self.append_type("Ser");
        self
    }

    pub fn split_for_impl_ipd(
        &mut self,
        used: &[&Ident],
    ) -> (ImplGenerics, TypeGenerics, Option<&WhereClause>) {
        self.mod_for_ipd(used);
        self.0.split_for_impl()
    }

    pub fn mod_for_esd(&mut self, used: &[&Ident]) -> &mut Self {
        let orig_where = self.0.where_clause.as_ref().map(|v| &v.predicates);
        let fetch = Self::gen_fetch_for_ty(used, quote!('t), quote!(Set), true);
        let new_where: WhereClause = parse_quote! {
            where
            Set: cote::prelude::SetValueFindExt,
            cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default,
            #fetch
            #orig_where
        };

        self.0.where_clause = Some(new_where);
        self.insert_lifetime("'set");
        self.append_type("Set");
        self
    }

    pub fn split_for_impl_esd(
        &mut self,
        used: &[&Ident],
    ) -> (ImplGenerics, TypeGenerics, Option<&WhereClause>) {
        self.mod_for_esd(used);
        self.0.split_for_impl()
    }

    pub fn mod_for_pi(&mut self, used: &[&Ident]) -> &mut Self {
        let orig_where = self.0.where_clause.as_ref().map(|v| &v.predicates);
        let new_where: WhereClause = parse_quote! {
            where
                #(#used: cote::prelude::Infer + cote::prelude::ErasedTy,)*
                #(<#used as cote::prelude::Infer>::Val: cote::prelude::RawValParser,)*
                #orig_where
        };

        self.0.where_clause = Some(new_where);
        self
    }

    pub fn split_for_impl_pi(
        &mut self,
        used: &[&Ident],
    ) -> (ImplGenerics, TypeGenerics, Option<&WhereClause>) {
        self.mod_for_pi(used);
        self.0.split_for_impl()
    }

    pub fn mod_for_fetch(&mut self, used: &[&Ident]) -> &mut Self {
        let orig_where = self.0.where_clause.as_ref().map(|v| &v.predicates);
        let fetch = Self::gen_fetch_for_ty(used, quote!('set), quote!(Set), false);
        let new_where: WhereClause = parse_quote! {
            where
                Set: cote::prelude::SetValueFindExt,
                cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default,
                Self: cote::prelude::ErasedTy + Sized,
                #fetch
                #orig_where
        };

        self.0.where_clause = Some(new_where);
        self.insert_lifetime("'set");
        self.append_type("Set");
        self
    }

    pub fn split_for_impl_fetch(
        &mut self,
        used: &[&Ident],
    ) -> (ImplGenerics, TypeGenerics, Option<&WhereClause>) {
        self.mod_for_fetch(used);
        self.0.split_for_impl()
    }

    pub fn gen_alter_for_ty(used: &[&Ident]) -> TokenStream {
        quote! {
            #(#used: cote::prelude::Alter,)*
        }
    }

    pub fn gen_fetch_for_ty(
        used: &[&Ident],
        lifetime: TokenStream,
        set: TokenStream,
        for_: bool,
    ) -> TokenStream {
        if for_ {
            quote! {
                #(#used: for <#lifetime> cote::prelude::Fetch<#lifetime, #set>,)*
            }
        } else {
            quote! {
                #(#used: cote::prelude::Fetch<#lifetime, #set>,)*
            }
        }
    }
}

impl ToTokens for GenericsModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        ToTokens::to_tokens(&self.0, tokens)
    }
}

#[derive(Debug, Default)]
pub struct OptUpdate {
    pub c: Option<TokenStream>,

    pub i: Option<TokenStream>,

    pub h: Option<TokenStream>,
}

impl OptUpdate {
    pub fn with_create(mut self, value: TokenStream) -> Self {
        self.c = Some(value);
        self
    }

    pub fn with_insert(mut self, value: TokenStream) -> Self {
        self.i = Some(value);
        self
    }

    pub fn with_handler(mut self, value: TokenStream) -> Self {
        self.h = Some(value);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WrapperTy<'a> {
    Opt(&'a Type),

    Vec(&'a Type),

    OptVec(&'a Type),

    Null(&'a Type),
}

impl<'a> WrapperTy<'a> {
    pub fn new(ty: &'a Type) -> Self {
        match Self::check_wrapper_ty(ty, "Option") {
            (true, inner_ty) => match Self::check_wrapper_ty(inner_ty, "Vec") {
                (true, inner_ty) => Self::OptVec(inner_ty),
                (false, inner_ty) => Self::Opt(inner_ty),
            },
            (false, inner_ty) => match Self::check_wrapper_ty(inner_ty, "Vec") {
                (true, inner_ty) => Self::Vec(inner_ty),
                (false, _) => Self::Null(ty),
            },
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null(_))
    }

    pub fn inner_type(&self) -> &Type {
        match self {
            Self::Opt(ty) => ty,
            Self::Vec(ty) => ty,
            Self::OptVec(ty) => ty,
            Self::Null(ty) => ty,
        }
    }

    pub fn check_wrapper_ty(ty: &'a Type, name: &str) -> (bool, &'a Type) {
        if let Type::Path(path) = ty {
            if let Some(segment) = path.path.segments.last() {
                let ident_str = segment.ident.to_string();

                if ident_str == name {
                    if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                        if let Some(GenericArgument::Type(next_ty)) = ab.args.first().as_ref() {
                            return (true, next_ty);
                        }
                    }
                }
            }
        }
        (false, ty)
    }
}
