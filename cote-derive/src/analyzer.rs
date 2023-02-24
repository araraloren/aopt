use std::ops::{Deref, DerefMut};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Data::Struct, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument,
    GenericParam, Generics, Lifetime, LifetimeDef, Path, PathArguments, Type, TypeArray, TypePath,
    TypeReference, TypeTuple, WhereClause,
};
use syn::{Token, TypeParam, WherePredicate};

use crate::global::{ArgCfg, Configurations, GlobalCfg};
use crate::global::{CfgKind, FieldCfg, SubCfg};

pub fn derive_parser(input: &DeriveInput) -> syn::Result<TokenStream> {
    let analyzer = Analyzer::new(input)?;
    let impl_for_parser = analyzer.generate_all()?;

    Ok(quote! {
        #impl_for_parser
    })
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

    pub fn generate_all(&self) -> syn::Result<TokenStream> {
        let has_handler_on_field = self
            .field_metas
            .iter()
            .any(|v| v.field_cfg.find_cfg(CfgKind::OptOn).is_some());
        let has_handler_on_global = self
            .struct_meta
            .global_cfg
            .find_cfg(CfgKind::ParserOn)
            .is_some();
        let update = self.generate_update()?;
        let extract_value = self.generate_try_extract()?;
        let try_from = self.generate_try_from()?;
        let ident = self.struct_meta.ident;
        let generics = &self.struct_meta.generics.params;
        let where_clause = self.struct_meta.generate_where_clause()?;
        let where_clause = if has_handler_on_field || has_handler_on_global {
            quote! {
                where
                P::Set: aopt::set::Set,
                P::Error: Into<aopt::Error>,
                P: aopt::parser::Policy + aopt::ext::APolicyExt<P> + Default,
                aopt::set::SetCfg<P::Set>: aopt::opt::Config + aopt::opt::ConfigValue + Default,
                P::Inv<'a>: aopt::ctx::HandlerCollection<'a, P::Set, P::Ser>,
                #where_clause
            }
        } else {
            quote! {
                where
                P::Set: aopt::set::Set,
                P::Error: Into<aopt::Error>,
                P: aopt::parser::Policy + aopt::ext::APolicyExt<P> + Default,
                aopt::set::SetCfg<P::Set>: aopt::opt::Config + aopt::opt::ConfigValue + Default,
                #where_clause
            }
        };

        if generics.is_empty() {
            Ok(quote! {
                impl<P> CoteParserDeriveExt<P> for #ident #where_clause
                {
                    fn update<'zlifetime>(parser: &mut aopt::parser::Parser<'zlifetime, P>) -> Result<(), aopt::Error> {
                        #update
                    }
                }

                #extract_value

                #try_from
            })
        } else {
            Ok(quote! {
                impl<#generics, P> CoteParserDeriveExt<P> for #ident<#generics> #where_clause
                {
                    fn update<'zlifetime>(parser: &mut aopt::parser::Parser<'zlifetime, P>) -> Result<(), aopt::Error> {
                        #update
                    }
                }

                #extract_value

                #try_from
            })
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
                    ctor.new_with({ #ctor_new_with }).map_err(Into::into)?
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
        ret.extend(self.struct_meta.generate_main()?);
        ret.extend(quote! { Ok(()) });
        Ok(ret)
    }

    pub fn generate_try_from(&self) -> syn::Result<TokenStream> {
        let generics = &self.struct_meta.generics.params;
        let where_clause = self.struct_meta.generate_where_clause()?;
        let ident = self.struct_meta.ident;
        let parser_ty = self.struct_meta.gen_parser_type()?;

        Ok(if generics.is_empty() {
            quote! {
                impl <'zlifetime> std::convert::TryFrom<&'zlifetime mut #parser_ty> for #ident {
                    type Error = aopt::Error;

                    fn try_from(parser: &'zlifetime mut #parser_ty) -> Result<Self, Self::Error> {
                        <#ident as CoteParserExtractValueExt<aopt::ext::ASet>>::try_extract(parser.optset_mut())
                    }
                }
            }
        } else {
            quote! {
                impl <'zlifetime, #generics> std::convert::TryFrom<&'zlifetime mut #parser_ty>
                    for #ident<#generics> where #where_clause {
                    type Error = aopt::Error;

                    fn try_from(parser: &'zlifetime mut #parser_ty) -> Result<Self, Self::Error> {
                        <#ident as CoteParserExtractValueExt<aopt::ext::ASet>>::try_extract(parser.optset_mut())
                    }
                }
            }
        })
    }

    pub fn generate_try_extract(&self) -> syn::Result<TokenStream> {
        let mut mut_field = quote! {};
        let mut ref_field = quote! {};
        let generics = &self.struct_meta.generics.params;
        let where_clause = self.struct_meta.generate_where_clause()?;
        let ident = self.struct_meta.ident;

        for field in self.field_metas.iter() {
            let (is_reference, code) = field.generate_try_extract()?;

            if is_reference {
                ref_field.extend(code);
            } else {
                mut_field.extend(code);
            }
        }
        Ok(if generics.is_empty() {
            quote! {
                impl <'zlifetime, S> CoteParserExtractValueExt<'zlifetime, S>
                    for #ident where S: aopt::set::SetValueFindExt, #where_clause {
                    fn try_extract(set: &'zlifetime mut S) -> Result<Self, aopt::Error> where Self: Sized {
                        Ok(Self {
                            #mut_field
                            #ref_field
                        })
                    }
                }
            }
        } else {
            quote! {
                impl <'zlifetime, #generics, S> CoteParserExtractValueExt<'zlifetime, S>
                    for #ident<#generics> where S: aopt::set::SetValueFindExt, #where_clause {
                    fn try_extract(set: &'zlifetime mut S) -> Result<Self, aopt::Error> where Self: Sized {
                        Ok(Self {
                            #mut_field
                            #ref_field
                        })
                    }
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct StructMeta<'a> {
    ident: &'a Ident,

    generics: &'a Generics,

    tys: Vec<&'a Ident>,

    lifetimes: Vec<&'a Ident>,

    where_clause: Option<&'a Punctuated<WherePredicate, Token!(,)>>,

    global_cfg: Configurations<GlobalCfg>,
}

impl<'a> StructMeta<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let params = &generics.params;
        let where_clause = generics.where_clause.as_ref().map(|v| &v.predicates);
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

    pub fn has_generics(&self) -> bool {
        !self.generics.params.is_empty()
    }

    pub fn generate_main(&self) -> syn::Result<TokenStream> {
        Ok(
            if let Some(cfg) = self.global_cfg.find_cfg(CfgKind::ParserOn) {
                let value = &cfg.value;

                quote! {
                    parser.add_opt_i::<Main>("default_main")?.on(#value)?;
                }
            } else {
                quote! {}
            },
        )
    }

    pub fn generate_where_clause(&self) -> syn::Result<TokenStream> {
        let mut code = quote! {};
        let zlifetime = Lifetime::new("'zlifetime", self.ident.span());

        for lifetime in self.lifetimes.iter() {
            let lifetime = Lifetime::new(&format!("'{}", lifetime.to_string()), lifetime.span());

            code.extend(quote! {
                #zlifetime: #lifetime,
            });
        }
        Ok(if let Some(where_clause) = self.where_clause {
            quote! { #code #where_clause }
        } else if !self.lifetimes.is_empty() {
            quote! { #code }
        } else {
            quote! {}
        })
    }

    pub fn gen_parser_type(&self) -> syn::Result<TokenStream> {
        let policy = self.global_cfg.find_cfg(CfgKind::ParserPolicy);
        let policy_name = policy
            .map(|v| v.value.to_token_stream().to_string())
            .unwrap_or(String::from("fwd"));

        Ok(match policy_name.as_str() {
            "pre" => {
                quote! {
                    aopt::ext::APreParser<'_>
                }
            }
            "fwd" => {
                quote! {
                    aopt::ext::AFwdParser<'_>
                }
            }
            "delay" => {
                quote! {
                    aopt::ext::ADelayParser<'_>
                }
            }
            _ => policy_name.to_token_stream(),
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
                "can not have both `arg` and `sub` on one field",
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

    pub fn generate_try_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let is_ref = self.field_cfg.find_cfg(CfgKind::OptRef).is_some();
        let is_mut = self.field_cfg.find_cfg(CfgKind::OptMut).is_some();
        let ident = self.ident.unwrap_or_else(|| {
            abort! {
                self.ident,
                "missing filed name",
            }
        });
        let name = format!("--{}", ident.to_string()).to_token_stream();
        let name = self
            .field_cfg
            .find_cfg(CfgKind::OptName)
            .map(|v| v.value.to_token_stream())
            .unwrap_or(name);

        if is_ref && is_mut {
            abort! {
                ident,
                "can not set both mut and ref on arg"
            }
        } else if is_ref {
            Ok((
                true,
                quote! {
                    #ident: aopt::value::InferValueRef::infer_fetch(#name, set)?,
                },
            ))
        } else if is_mut {
            Ok((
                false,
                quote! {
                    #ident: aopt::value::InferValueMut::infer_fetch(#name, set)?,
                },
            ))
        } else if self.is_reference {
            Ok((
                true,
                quote! {
                    #ident: aopt::value::InferValueRef::infer_fetch(#name, set)?,
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: aopt::value::InferValueMut::infer_fetch(#name, set)?,
                },
            ))
        }
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
            let mut config = aopt::set::SetCfg::<P::Set>::default();
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
