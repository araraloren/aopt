use proc_macro2::TokenStream;
use quote::{quote};
use syn::{DeriveInput, Data::Struct, DataStruct, Fields, Type, TypePath, PathArguments, AngleBracketedGenericArguments, GenericArgument};

use crate::global::GlobalCfgs;

pub fn derive_parser(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let cfgs = GlobalCfgs::parse_attrs(&input.attrs);
    
    match input.data {
        Struct(
            DataStruct {
                fields: Fields::Named(ref fields),
                ..
            }
        ) => {
            for field in fields.named.iter() {
                print!(":: --> got a field {:?}: \n", field.ident);
                display_of_ty(&field.ty);
                println!("");
            }
        }
        _ => {}
    }

    quote! {}
}

fn display_of_ty(mut ty: &Type) {
    while let syn::Type::Group( syn::TypeGroup { elem, ..} )  = ty {
        ty = elem;
    }   
    match ty {
        Type::Reference(tyref) => {
            dbg!(tyref);
        },
        Type::Path(TypePath { path, .. }) => {
            let mut segment = path.segments.last();
            let mut last_ident = None;
            let mut last_type = path;

            while let Some(seg) = segment {
                println!("current = --> {}", seg.ident);
                last_ident = Some(&seg.ident);
                match &seg.arguments {
                    PathArguments::AngleBracketed(args) => {
                        let args = &args.args;
                        let arg = &args[0];
                        
                        segment = None;
                        if let GenericArgument::Type(path) = arg {
                            if let Type::Path( TypePath { path, .. }) = path {
                                segment = path.segments.last();
                                last_type = path;
                            }
                        }
                    },
                    _ => {
                        segment = None
                    }
                }
            }
            println!("--> LAST TYPE = {:?}", last_ident);
            println!("--> LAST TYPE = {:?}", last_type);
        }
        _ => {
            println!("NOT SUPPORT: {:?}", ty);
        },
    }
}