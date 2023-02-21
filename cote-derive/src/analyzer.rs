use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{
    Data::Struct, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument,
    GenericParam, Generics, Lifetime, LifetimeDef, Path, PathArguments, Type, TypePath,
    TypeReference, WhereClause,
};

///
/// pub struct Widget<'a, T> {
///     a: &'a T,
///     b: &'a mut Option<T>,
///     c: Option<&'a T>,
/// }
pub fn derive_parser(input: DeriveInput) -> TokenStream {
    let analyzer = Analyzer::new(&input).unwrap_or_else(|e| {
        abort! {
            input, "parsing struct failed: {:?}", e
        }
    });
    let generics = analyzer.struct_meta.generics;
    let ident = analyzer.struct_meta.ident;
    let where_clause = analyzer.struct_meta.where_clause;

    for field in analyzer.field_metas {
        if let Some(ident) = field.ident {
            let ty = field.ty;
            let trimed_ty = field.trimed_ty;

            println!(
                "get ident = {}, ty = {}, trimed = {}",
                ident.to_string(),
                ty.to_token_stream().to_string(),
                trimed_ty.to_token_stream().to_string()
            );
        } else {
            panic!("???????????? {:?}", field)
        }
    }
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
                    field_metas.push(FieldMeta::new(&struct_meta, field)?);
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
}

impl<'a> StructMeta<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let params = &generics.params;
        let where_clause = generics.where_clause.as_ref();
        let mut lifetimes = vec![];
        let mut tys = vec![];

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
            where_clause,
        })
    }
}

#[derive(Debug)]
pub struct FieldMeta<'a> {
    ident: Option<&'a Ident>,

    ty: &'a Type,

    trimed_ty: Type,

    has_lifetime: bool,
}

impl<'a> FieldMeta<'a> {
    pub fn new(major_meta: &StructMeta<'a>, field: &'a Field) -> syn::Result<Self> {
        let ident = field.ident.as_ref();
        let ty = &field.ty;
        let (has_lifetime, trimed_ty) = Self::trim_ty(ty);

        Ok(Self {
            ident,

            ty,

            trimed_ty,

            has_lifetime,
        })
    }

    pub fn trim_ty(ty: &Type) -> (bool, Type) {
        let mut ty = ty.clone();
        let mut has_lifetime = false;

        if let Type::Reference(tr) = &mut ty {
            tr.lifetime = None;
            has_lifetime = true;
        } else if let Type::Path(tp) = &mut ty {
            let mut tp = Some(tp);

            let sgment = tp.path.segments.last_mut();

            if let Some(mut segment) = sgment {
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

        (has_lifetime, ty)
    }
}
