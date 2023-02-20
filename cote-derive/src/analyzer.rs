
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{
    Data::Struct, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, Path,
    PathArguments, Type, TypePath, TypeReference, GenericParam, Lifetime, LifetimeDef, Generics, WhereClause,
};


///
/// pub struct Widget<'a, T> {
///     a: &'a T,
///     b: &'a mut Option<T>,
///     c: Option<&'a T>,
/// }
pub fn derive_parser(input: DeriveInput) -> TokenStream {
    let analyzer = Analyzer::new(&input);

    dbg!(analyzer);
    quote!{

    }
}

#[derive(Debug)]
pub struct Analyzer<'a> {
    struct_meta: StructMeta<'a>,
}

impl<'a> Analyzer<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        Ok(Self {
            struct_meta: StructMeta::new(input)?,
        })
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
                },
                GenericParam::Lifetime(lifetime) => {
                    lifetimes.push(&lifetime.lifetime.ident);
                },
                GenericParam::Const(const_param) => {
                    abort! {
                        input,
                        "analyzer struct failed: Cote not support const parameter `{:?}`",
                        const_param,
                    }
                },
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