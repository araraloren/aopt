pub mod alter;
pub mod arg;
pub mod cote;
pub mod fetch;
pub mod infer;
pub mod sub;
pub mod utils;
pub mod value;

use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Field;
use syn::GenericArgument;
use syn::Lit;
use syn::PathArguments;
use syn::Type;

pub use self::utils::*;

use crate::error;
use crate::value::Value;

pub use self::alter::AlterGenerator;
pub use self::cote::CoteGenerator;
pub use self::fetch::FetchGenerator;
pub use self::infer::InferGenerator;
pub use self::value::ValueGenerator;

pub fn gen_option_ident(idx: usize, span: Span) -> Ident {
    Ident::new(&format!("option_{}", idx), span)
}

pub fn gen_option_uid_ident(idx: usize, span: Span) -> Ident {
    Ident::new(&format!("option_uid_{}", idx), span)
}

pub fn check_if_has_sub_cfg(field: &Field) -> syn::Result<bool> {
    let attrs = &field.attrs;
    let has_sub_cfg = attrs.iter().any(|v| v.path.is_ident("sub"));
    let has_arg_cfg = attrs.iter().any(|v| v.path.is_ident(CONFIG_ARG));
    let has_cmd_cfg = attrs.iter().any(|v| v.path.is_ident(CONFIG_CMD));
    let has_pos_cfg = attrs.iter().any(|v| v.path.is_ident(CONFIG_POS));

    if (has_arg_cfg || has_cmd_cfg || has_pos_cfg) && has_sub_cfg {
        Err(error(
            field,
            "can not have both `sub` and `arg` configuration on same field".to_owned(),
        ))
    } else {
        Ok(has_sub_cfg)
    }
}

pub fn gen_ret_default_policy_ty(policy_name: &str, policy: Option<&Value>) -> Option<TokenStream> {
    match policy_name {
        POLICY_PRE => Some(quote! {
            cote::PrePolicy<'inv, cote::ASet, cote::ASer>
        }),
        POLICY_FWD => Some(quote! {
            cote::FwdPolicy<'inv, cote::ASet, cote::ASer>
        }),
        POLICY_DELAY => Some(quote! {
            cote::DelayPolicy<'inv, cote::ASet, cote::ASer>
        }),
        _ => {
            let value = policy.unwrap();

            Some(quote! {
                #value<'inv, cote::ASet, cote::ASer>
            })
        }
    }
}

pub fn gen_policy_ty_generics(policy_name: &str, policy: Option<&Value>) -> Option<TokenStream> {
    match policy_name {
        POLICY_PRE => Some(quote! {
            cote::PrePolicy::<'inv, Set, Ser>
        }),
        POLICY_FWD => Some(quote! {
            cote::FwdPolicy::<'inv, Set, Ser>
        }),
        POLICY_DELAY => Some(quote! {
            cote::DelayPolicy::<'inv, Set, Ser>
        }),
        _ => {
            let value = policy.unwrap();

            Some(quote! {
                #value::<'inv, Set, Ser>
            })
        }
    }
}

pub fn gen_ret_policy_ty_generics(
    policy_name: &str,
    policy: Option<&Value>,
) -> Option<TokenStream> {
    match policy_name {
        POLICY_PRE => Some(quote! {
            cote::PrePolicy<'inv, Set, Ser>
        }),
        POLICY_FWD => Some(quote! {
            cote::FwdPolicy<'inv, Set, Ser>
        }),
        POLICY_DELAY => Some(quote! {
            cote::DelayPolicy<'inv, Set, Ser>
        }),
        _ => {
            let value = policy.unwrap();

            Some(quote! {
                #value<'inv, Set, Ser>
            })
        }
    }
}

pub fn filter_comment_doc(attrs: &[Attribute]) -> Vec<Lit> {
    let attrs = attrs.iter().filter(|v| v.path.is_ident("doc"));
    let mut ret = vec![];

    for attr in attrs {
        if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
            if let syn::Lit::Str(_) = &meta.lit {
                ret.push(meta.lit);
            }
        }
    }
    ret
}

pub fn check_in_path(ty: &Type, name: &str) -> syn::Result<bool> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            let ident = segment.ident.to_string();

            if ident == name {
                return Ok(true);
            } else if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                for arg in ab.args.iter() {
                    if let GenericArgument::Type(next_ty) = arg {
                        return check_in_path(next_ty, name);
                    }
                }
            }
        }
        Ok(false)
    } else {
        Err(error(ty, "Cote not support reference type".to_owned()))
    }
}

pub fn gen_ty_without_option(ty: &Type) -> syn::Result<Type> {
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

pub fn gen_subapp_without_option(ty: &Type) -> syn::Result<&Ident> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return Ok(&segment.ident);
        }
    }
    Err(error(ty, "can not generate sub app type".to_owned()))
}
