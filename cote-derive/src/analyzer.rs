use std::ops::{Deref, DerefMut};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{
    Data::Struct, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument,
    GenericParam, Generics, Lifetime, LifetimeDef, Path, PathArguments, Type, TypeArray, TypePath,
    TypeReference, TypeTuple, WhereClause,
};

use crate::global::{Configurations, FieldCfg, GlobalCfg};

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
        let global_cfg = Configurations::<GlobalCfg>::parse_attrs(Some(ident), &input.attrs);

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
        let field_cfg = Configurations::<FieldCfg>::parse_attrs(ident, &field.attrs);

        Ok(Self {
            ident,
            ty,
            trimed_ty,
            field_cfg,
            is_reference,
        })
    }
}

pub fn remove_lifetime(ty: &Type) -> (bool, Type) {
    let mut ty = ty.clone();
    let mut is_reference = false;

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
