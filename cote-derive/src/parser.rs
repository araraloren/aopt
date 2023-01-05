use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse::Parse, Data::Struct, DataStruct, DeriveInput, Field, Fields, GenericArgument, Path,
    PathArguments, Type, TypePath, TypeReference,
};

use crate::global::GlobalCfgs;

pub fn derive_parser(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let cfgs = GlobalCfgs::parse_attrs(&input.attrs);

    dbg!(&cfgs);
    match input.data {
        Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => {
            for field in fields.named.iter() {
                if let Some(field) = CoteField::parse_field(field) {
                    eprintln!(
                        "got a field => {:?} mut = {}, ref = {}",
                        field.ident, field.mutable, field.reference
                    );
                    for path in field.paths {
                        eprintln!(
                            ":: path --> mut = {}, ref = {}, ident = {:?}, path = {:?}",
                            path.mutable, path.reference, path.ident, path.path
                        );
                    }
                }
            }
        }
        _ => {}
    }

    quote! {}
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
