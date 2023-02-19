use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{
    Data::Struct, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, Path,
    PathArguments, Type, TypePath, TypeReference, GenericParam,
};

use crate::{
    lifetime_gen::LifetimedCodeGenerator,
    global::{CfgKind, Configurations, FieldCfg, GlobalCfg},
};

pub(crate) struct FieldInfo {
    pub has_lifetime: bool,

    pub trimed_ty: Type,
}

pub fn derive_parser(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let global_cfgs = Configurations::<GlobalCfg>::parse_attrs(Some(ident), &input.attrs);
    let parameters = input.generics.params.to_token_stream();
    let mut lifetime = None;

    for param in input.generics.params.iter() {
        if let GenericParam::Lifetime(lf) = param {
            lifetime = Some(lf.lifetime.clone());
        }
    }
    match input.data {
        Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            let mut fields_ = vec![];

            for field in fields.named.iter() {
                if let Some(field) = CoteField::parse_field(field) {
                    eprintln!(
                        "got a field => {:?} mut = {}, ref = {}",
                        field.ident, field.mutable, field.reference
                    );
                    for (idx, path) in field.paths.iter().enumerate() {
                        eprintln!(
                            ":: path @ {idx} --> mut = {}, ref = {}, ident = {:?}, path = {:?}",
                            path.mutable, path.reference, path.ident, path.path
                        );
                    }
                }
                let mut ty = field.ty.clone();
                let mut has_lifetime = false;

                if let Type::Reference(tr) = &mut ty {
                    tr.lifetime = None;
                    has_lifetime = true;
                } else if let Type::Path(tp) = &mut ty {
                    for segment in tp.path.segments.iter_mut() {
                        if let PathArguments::AngleBracketed(ab) = &mut segment.arguments {
                            for arg in ab.args.iter_mut() {
                                if let GenericArgument::Type(Type::Reference(tr)) = arg {
                                    tr.lifetime = None;
                                    has_lifetime = true;
                                }
                            }
                        }
                    }
                }
                fields_.push((
                    field,
                    Configurations::<FieldCfg>::parse_attrs(field.ident.as_ref(), &field.attrs),
                    FieldInfo {
                        trimed_ty: ty,
                        has_lifetime,
                    }
                ));
            }

            if let Some(lifetime) = lifetime {
                let generator = LifetimedCodeGenerator {
                    ident,
                    global_cfg: global_cfgs,
                    fields: fields_,
                };
    
                generator.generate(parameters, lifetime).unwrap_or_else(|e| {
                    abort! {
                        ident, "can not generate code: {:?}", e
                    }
                })
            }
            else {
                quote!{}
            }
        }
        Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => {
            quote! {}
        }
        _ => {
            quote! {}
        }
    }
}

#[derive(Debug, Clone)]
struct CoteTy<'a> {
    pub ident: Option<&'a Ident>,

    pub path: &'a Path,

    pub mutable: bool,

    pub reference: bool,
}

impl<'a> CoteTy<'a> {
    fn parse_ty(input: &'a Type) -> Option<Self> {
        match input {
            Type::Path(TypePath { path, .. }) => {
                let ident = path.segments.last().map(|v| &v.ident);

                Some(Self {
                    ident,
                    path,
                    mutable: false,
                    reference: false,
                })
            }
            Type::Reference(TypeReference {
                mutability, elem, ..
            }) => {
                let mut ty = Self::parse_ty(&elem);

                if let Some(ty) = ty.as_mut() {
                    ty.mutable = mutability.is_some();
                    ty.reference = true;
                }
                ty
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct CoteField<'a> {
    ident: Option<&'a Ident>,

    mutable: bool,

    reference: bool,

    paths: Vec<CoteTy<'a>>,
}

impl<'a> CoteField<'a> {
    pub fn parse_field(input: &'a Field) -> Option<Self> {
        let ident = input.ident.as_ref();
        let mut ty = &input.ty;

        while let syn::Type::Group(syn::TypeGroup { elem, .. }) = ty {
            ty = elem;
        }
        Self::parse_ty(ty).map(|(mut_, ref_, ty_)| Self {
            ident,
            mutable: mut_,
            reference: ref_,
            paths: ty_,
        })
    }

    pub fn parse_ty(input: &'a Type) -> Option<(bool, bool, Vec<CoteTy<'a>>)> {
        match input {
            Type::Path(TypePath { path, .. }) => {
                let mut paths = vec![];
                let mut segment = path.segments.last();

                paths.push(CoteTy::parse_ty(input).unwrap());
                loop {
                    if let Some(seg) = segment {
                        match &seg.arguments {
                            PathArguments::AngleBracketed(args) => {
                                let args = &args.args;
                                let arg = &args.last();

                                segment = None;
                                if let Some(GenericArgument::Type(path)) = arg {
                                    let cote_ty = CoteTy::parse_ty(path).unwrap();

                                    segment = cote_ty.path.segments.last();
                                    paths.push(cote_ty);
                                }
                            }
                            _ => segment = None,
                        }
                    } else {
                        break;
                    }
                }
                Some((false, false, paths))
            }
            Type::Reference(TypeReference {
                mutability, elem, ..
            }) => Self::parse_ty(&elem).map(|(_, _, ty_)| (mutability.is_some(), true, ty_)),
            _ => None,
        }
    }
}
