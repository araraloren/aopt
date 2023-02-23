use std::ops::{Deref, DerefMut};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{
    Data::Struct, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument,
    GenericParam, Generics, Lifetime, LifetimeDef, Path, PathArguments, Type, TypeArray, TypePath,
    TypeReference, TypeTuple, WhereClause,
};

use crate::global::{ArgCfg, Configurations, GlobalCfg};
use crate::global::{CfgKind, FieldCfg, SubCfg};

pub fn derive_parser(input: DeriveInput) -> TokenStream {
    let analyzer = Analyzer::new(&input).unwrap_or_else(|e| {
        abort! {
            input, "parsing struct failed: {:?}", e
        }
    });
    let generics = analyzer.struct_meta.generics;
    let ident = analyzer.struct_meta.ident;
    let where_clause = analyzer.struct_meta.where_clause;

    dbg!(&analyzer);
    quote! {
        impl #generics You for #ident #generics #where_clause {
            fn you(&self) {
                println!("New implement for {}", stringify!(#ident));
            }
        }
    }
}

#[derive(Debug)]
pub struct Analyzer<'a> {
    struct_meta: StructMeta<'a>,

    field_metas: Vec<FieldMeta<'a>>,
}

impl<'a> Analyzer<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        match input.data {
            Struct(DataStruct {
                fields: Fields::Named(ref fields),
                ..
            }) => {
                let struct_meta = StructMeta::new(input)?;
                let mut field_metas = vec![];

                for field in fields.named.iter() {
                    field_metas.push(FieldMeta::new(field)?);
                }
                Ok(Self {
                    field_metas,
                    struct_meta,
                })
            }
            _ => {
                abort! {
                    input,
                        "cote only support struct format"
                }
            }
        }
    }

    pub fn generate_update(&self) -> syn::Result<TokenStream> {
        let mut ret = quote! {
            let set = parser.optset_mut();
            let ctor_name = aopt::prelude::ctor_default_name();
            let ctor = set.ctor_mut(&ctor_name)?;
        };
        let mut configs = vec![];
        let mut inserts = vec![];
        let mut handlers = vec![];

        for (idx, field) in self.field_metas.iter().enumerate() {
            let ident = Ident::new(&format!("option{}", idx), field.ident.span());
            let ctor_new_with = field.generate_config()?;
            let handler_cfg = field.field_cfg.find_cfg(CfgKind::OptOn);

            configs.push(quote! {
                let #ident = {
                    #ctor_new_with
                };
            });
            if let Some(cfg) = handler_cfg {
                let uid_ident = Ident::new(&format!("option_uid_{}", idx), field.ident.span());
                let handler = cfg.value.to_token_stream();

                inserts.push(quote! {
                    let #uid_ident = set.insert(#ident);
                });
                handlers.push(quote! {
                    parser.entry(#uid_ident).on(#handler)?;
                });
            } else {
                inserts.push(quote! {
                    set.insert(#ident);
                });
            }
        }
        ret.extend(configs.into_iter());
        ret.extend(inserts.into_iter());
        ret.extend(handlers.into_iter());
        ret.extend(quote! { Ok(()) });
        Ok(ret)
    }
}

#[derive(Debug)]
pub struct StructMeta<'a> {
    ident: &'a Ident,

    generics: &'a Generics,

    tys: Vec<&'a Ident>,

    lifetimes: Vec<&'a Ident>,

    where_clause: Option<&'a WhereClause>,

    global_cfg: Configurations<GlobalCfg>,
}

impl<'a> StructMeta<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let params = &generics.params;
        let where_clause = generics.where_clause.as_ref();
        let mut lifetimes = vec![];
        let mut tys = vec![];
        let global_cfg =
            Configurations::<GlobalCfg>::parse_attrs(Some(ident), &input.attrs, "cote");

        for param in params {
            match param {
                GenericParam::Type(ty) => {
                    tys.push(&ty.ident);
                }
                GenericParam::Lifetime(lifetime) => {
                    lifetimes.push(&lifetime.lifetime.ident);
                }
                GenericParam::Const(const_param) => {
                    abort! {
                        input,
                        "analyzer struct failed: Cote not support const parameter `{:?}`",
                        const_param,
                    }
                }
            }
        }
        Ok(Self {
            tys,
            ident,
            generics,
            lifetimes,
            global_cfg,
            where_clause,
        })
    }
}

#[derive(Debug)]
pub struct FieldMeta<'a> {
    ident: Option<&'a Ident>,

    ty: &'a Type,

    trimed_ty: Type,

    is_reference: bool,

    field_cfg: Configurations<FieldCfg>,
}

impl<'a> FieldMeta<'a> {
    pub fn new(field: &'a Field) -> syn::Result<Self> {
        let ident = field.ident.as_ref();
        let ty = &field.ty;
        let (is_reference, trimed_ty) = remove_lifetime(ty);
        let arg_cfg = Configurations::<ArgCfg>::parse_attrs(ident, &field.attrs, "arg");
        let sub_cfg = Configurations::<SubCfg>::parse_attrs(ident, &field.attrs, "sub");

        let field_cfg = if arg_cfg.cfgs.len() > 0 && sub_cfg.cfgs.len() > 0 {
            abort! {
                ident,
                "can not both `arg` and `sub` on one field",
            }
        } else if arg_cfg.cfgs.len() > 0 {
            Configurations {
                cfgs: arg_cfg.cfgs.into_iter().map(|v| v.into()).collect(),
            }
        } else {
            Configurations {
                cfgs: sub_cfg.cfgs.into_iter().map(|v| v.into()).collect(),
            }
        };

        Ok(Self {
            ident,
            ty,
            trimed_ty,
            field_cfg,
            is_reference,
        })
    }

    pub fn generate_option(&self) -> syn::Result<TokenStream> {
        let config = self.generate_config()?;

        Ok(quote! {
            let config = { #config };
            ctor.new_with(config).map_err(Into::into)?
        })
    }

    pub fn generate_config(&self) -> syn::Result<TokenStream> {
        let ty = self.ty;
        let ident = self.ident;
        let config = &self.field_cfg;
        let trimed_ty = &self.trimed_ty;

        let mut codes = vec![];
        let mut has_name = false;
        let mut ret = quote! {
            let mut config = SetCfg::<P::Set>::default();
        };

        for cfg in config.cfgs.iter() {
            codes.push(match cfg.kind {
                CfgKind::OptHint => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_hint(#token);
                    }
                }
                CfgKind::OptHelp => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_help(#token);
                    }
                }
                CfgKind::OptName => {
                    let token = cfg.value.to_token_stream();

                    has_name = true;
                    quote! {
                        config.set_name(#token);
                    }
                }
                CfgKind::OptValue => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_initializer(aopt::value::ValInitializer::new_value(<<#ty as aopt::value::Infer>::Val>::from(#token)));
                    }
                }
                CfgKind::OptValues => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        let values = #token.into_iter().map(|v|<<#ty as aopt::value::Infer>::Val>::from(v)).collect::<Vec<<#ty as aopt::value::Infer>::Val>>();
                        config.set_initializer(aopt::value::ValInitializer::new_values(values));
                    }
                }
                CfgKind::OptAlias => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.add_alias(#token);
                    }
                }
                CfgKind::OptAction => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_action(#token);
                    }
                }
                CfgKind::OptIndex => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_index(aopt::opt::Index::parse(#token)?);
                    }
                }
                CfgKind::OptValidator => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_storer(aopt::value::ValStorer::new_validator::<#ty>(#token));
                    }
                }
                CfgKind::OptOn | CfgKind::OptRef | CfgKind::OptMut => {
                    // will process in another function
                    quote! { }
                }
                _ => {
                    abort! {
                        ident, "Unsupport config kind on field: {:?}", cfg.kind
                    }
                }
            });
        }
        if !has_name {
            let ident = ident.map(|v| v.to_string()).unwrap_or_else(|| {
                abort! {
                    ident,
                    "missing field name for field {:?}", ident
                }
            });
            let name = format!(
                "{}{}",
                if ident.chars().count() > 1 { "--" } else { "-" },
                ident
            );

            codes.push(quote! {
                config.set_name(#name);
            });
        }
        codes.push(quote! {
            <#trimed_ty>::infer_fill_info(&mut config, true);
            config
        });
        ret.extend(codes.into_iter());
        Ok(ret)
    }
}

pub fn remove_lifetime(ty: &Type) -> (bool, Type) {
    let mut ty = ty.clone();
    let is_reference;

    if let Type::Reference(reference) = &mut ty {
        is_reference = true;
        remove_reference_lifetime(reference);
    } else {
        is_reference = check_if_reference(&ty);
        if let Type::Path(path) = &mut ty {
            remove_path_lifetime(path);
        }
    }
    (is_reference, ty)
}

pub fn check_if_reference(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => {
            if let Some(segment) = path.path.segments.last() {
                match &segment.arguments {
                    PathArguments::AngleBracketed(ab) => {
                        for arg in ab.args.iter() {
                            if let GenericArgument::Type(next_ty) = arg {
                                return check_if_reference(next_ty);
                            }
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        Type::Reference(_) => true,
        _ => false,
    }
}

pub fn remove_reference_lifetime(ty: &mut TypeReference) {
    ty.lifetime = None;
    match ty.elem.deref_mut() {
        Type::Path(path) => remove_path_lifetime(path),
        Type::Reference(ref_) => remove_reference_lifetime(ref_),
        _ => {
            // do nothing
        }
    }
}

pub fn remove_path_lifetime(ty: &mut TypePath) {
    if let Some(segment) = ty.path.segments.last_mut() {
        if let PathArguments::AngleBracketed(ab) = &mut segment.arguments {
            for arg in ab.args.iter_mut() {
                if let GenericArgument::Type(ty) = arg {
                    match ty {
                        Type::Path(path) => remove_path_lifetime(path),
                        Type::Reference(ref_) => remove_reference_lifetime(ref_),
                        _ => {}
                    };
                }
            }
        }
    }
}
