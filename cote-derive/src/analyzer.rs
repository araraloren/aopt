use std::ops::DerefMut;

use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Data::Struct;
use syn::DataStruct;
use syn::DeriveInput;
use syn::Field;
use syn::Fields;
use syn::GenericArgument;
use syn::GenericParam;
use syn::Generics;
use syn::Lifetime;
use syn::Lit;
use syn::PathArguments;
use syn::Token;
use syn::Type;
use syn::TypePath;
use syn::TypeReference;
use syn::WherePredicate;

use crate::global::ArgCfg;
use crate::global::CfgKind;
use crate::global::Configurations;
use crate::global::FieldCfg;
use crate::global::GlobalCfg;
use crate::global::SubCfg;

const HELP_OPTION_UID: &str = "help_option_uid";

pub fn derive_parser(input: &DeriveInput) -> syn::Result<TokenStream> {
    let analyzer = Analyzer::new(input)?;
    let impl_for_parser = analyzer.generate_simple_parser()?;

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

    pub fn generate_simple_parser(&self) -> syn::Result<TokenStream> {
        let has_handler_on_field = self.field_metas.iter().any(|v| v.has_handler());
        let has_handler_on_global = self.struct_meta.has_handler();
        let update = self.generate_parser_update()?;
        let extract_value = self.generate_try_extract()?;
        let try_from = self.generate_try_from()?;
        let ident = self.struct_meta.ident;
        let generics = &self.struct_meta.generics.params;
        let where_clause = self.struct_meta.generate_where_clause()?;
        let where_clause = if has_handler_on_field || has_handler_on_global {
            quote! {
                where
                P: 'zlifetime,
                P::Ser: 'zlifetime,
                P::Set: aopt::prelude::Set + 'zlifetime,
                P::Error: Into<aopt::Error>,
                P: aopt::prelude::Policy + aopt::prelude::APolicyExt<P> + Default,
                aopt::prelude::SetCfg<P::Set>: aopt::prelude::Config + aopt::prelude::ConfigValue + Default,
                P::Inv<'zlifetime>: aopt::ctx::HandlerCollection<'zlifetime, P::Set, P::Ser>,
                #where_clause
            }
        } else {
            quote! {
                where
                P: 'zlifetime,
                P::Ser: 'zlifetime,
                P::Set: aopt::prelude::Set + 'zlifetime,
                P::Error: Into<aopt::Error>,
                P: aopt::prelude::Policy + aopt::prelude::APolicyExt<P> + Default,
                aopt::prelude::SetCfg<P::Set>: aopt::prelude::Config + aopt::prelude::ConfigValue + Default,
                #where_clause
            }
        };
        let parse_interface = self.generate_parse_interface()?;

        if generics.is_empty() {
            Ok(quote! {
                impl<'zlifetime, P> cote::IntoParserDerive<'zlifetime, P> for #ident #where_clause
                {
                    fn update(parser: &mut aopt::prelude::Parser<'zlifetime, P>) -> Result<(), aopt::Error> {
                        #update
                    }
                }

                #extract_value

                #try_from

                #parse_interface
            })
        } else {
            Ok(quote! {
                impl<'zlifetime, #generics, P> cote::IntoParserDerive<'zlifetime, P> for #ident<#generics> #where_clause
                {
                    fn update(parser: &mut aopt::prelude::Parser<'zlifetime, P>) -> Result<(), aopt::Error> {
                        #update
                    }
                }

                #extract_value

                #try_from

                #parse_interface
            })
        }
    }

    pub fn generate_parse_interface(&self) -> syn::Result<TokenStream> {
        let generics = &self.struct_meta.generics.params;
        let ident = self.struct_meta.ident;
        let parser_ty = self.struct_meta.gen_parser_type()?;
        let where_clause = self.struct_meta.generate_where_clause()?;
        let inner = quote! {
            fn parse(args: aopt::ARef<aopt::prelude::Args>) -> Result<Self, aopt::Error> {
                let mut parser: #parser_ty = Self::into_parser()?;

                parser.init()?;
                match parser.parse(args).map_err(Into::into)?.ok() {
                    Ok(_) => {
                        Self::try_extract(parser.optset_mut())
                    }
                    Err(e) => {
                        Err(aopt::Error::raise_error(format!("parsing arguments failed: {:?}", e)))
                    }
                }
            }

            fn parse_env() -> Result<Self, aopt::Error> {
                Self::parse(aopt::ARef::new(aopt::prelude::Args::from_env()))
            }
        };

        Ok(if generics.is_empty() {
            quote! {
                impl #ident {
                    #inner
                }
            }
        } else {
            quote! {
                impl <#generics> #ident<#generics> where #where_clause {
                    #inner
                }
            }
        })
    }

    pub fn generate_parser_update(&self) -> syn::Result<TokenStream> {
        let mut ret = quote! {
            let set = parser.optset_mut();
            let ctor_name = aopt::prelude::ctor_default_name();
            let ctor = set.ctor_mut(&ctor_name)?;
        };
        let mut configs = vec![];
        let mut inserts = vec![];
        let mut handlers = vec![];
        let process_help = self
            .struct_meta
            .global_cfg
            .find_cfg(CfgKind::ParserHelp)
            .is_some();

        // insert main if global has `on`
        if let Some(cfg) = self.struct_meta.global_cfg.find_cfg(CfgKind::ParserOn) {
            let value = &cfg.value;
            let ident = Ident::new("main_option", self.struct_meta.ident.span());
            let uid_ident = Ident::new("main_option_uid", self.struct_meta.ident.span());

            configs.push(quote! {
                let #ident = {
                    ctor.new_with({
                        let mut config = aopt::prelude::SetCfg::<P::Set>::default();
                        config.set_name("main_option");
                        <aopt::opt::Main>::infer_fill_info(&mut config, true);
                        config
                    }).map_err(Into::into)?
                };
            });
            inserts.push(quote! {
                let #uid_ident = set.insert(#ident);
            });
            handlers.push(quote! {
                parser.entry(#uid_ident)?.on(#value);
            });
        }
        // insert help function
        if process_help {
            let head =
                if let Some(head_cfg) = self.struct_meta.global_cfg.find_cfg(CfgKind::ParserHead) {
                    let value = &head_cfg.value;

                    quote! {
                        #value
                    }
                } else {
                    quote! {
                        format!("{}", env!("CARGO_PKG_DESCRIPTION"))
                    }
                };
            let foot = if let Some(head_cfg) =
                self.struct_meta.global_cfg.find_cfg(CfgKind::ParserFoot)
            {
                let value = &head_cfg.value;

                quote! {
                    #value
                }
            } else {
                quote! {
                    format!("Create by {} v{}", env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_VERSION"))
                }
            };
            let name =
                if let Some(head_cfg) = self.struct_meta.global_cfg.find_cfg(CfgKind::ParserName) {
                    let value = &head_cfg.value;

                    quote! {
                        #value
                    }
                } else {
                    quote! {
                        format!("{}", env!("CARGO_PKG_NAME"))
                    }
                };
            let ident = Ident::new("help_option", self.struct_meta.ident.span());
            let uid_ident = Ident::new(HELP_OPTION_UID, self.struct_meta.ident.span());
            let width = if let Some(head_cfg) = self
                .struct_meta
                .global_cfg
                .find_cfg(CfgKind::ParserHelpWidth)
            {
                let value = &head_cfg.value;

                quote! {
                    #value
                }
            } else {
                quote! { 20 }
            };
            let usage_width = if let Some(head_cfg) = self
                .struct_meta
                .global_cfg
                .find_cfg(CfgKind::ParserUsageWidth)
            {
                let value = &head_cfg.value;

                quote! {
                    #value
                }
            } else {
                quote! { 10 }
            };

            configs.push(quote! {
                let #ident = {
                    ctor.new_with({
                        let mut config = aopt::prelude::SetCfg::<P::Set>::default();
                        config.set_name("--help");
                        config.add_alias("-h");
                        config.add_alias("-?");
                        config.set_help("Display help message");
                        <bool>::infer_fill_info(&mut config, true);
                        config
                    }).map_err(Into::into)?
                };
            });
            inserts.push(quote! {
                let #uid_ident = set.insert(#ident);
            });
            let main_ident = Ident::new("help_main_handler", self.struct_meta.ident.span());
            let main_uid_ident = Ident::new("help_main_handler_uid", self.struct_meta.ident.span());

            configs.push(quote! {
                let #main_ident = {
                    ctor.new_with({
                        let mut config = aopt::prelude::SetCfg::<P::Set>::default();
                        config.set_name("help_main_handler");
                        <aopt::opt::Main>::infer_fill_info(&mut config, true);
                        config
                    }).map_err(Into::into)?
                };
            });
            inserts.push(quote! {
                let #main_uid_ident = set.insert(#main_ident);
            });
            handlers.push(quote! {
                parser.entry(#main_uid_ident)?.on(
                    move |set: &mut P::Set, _: &mut P::Ser| -> Result<Option<()>, Error> {
                        let help_uid = #uid_ident;
                        if let Ok(value) = set.opt(help_uid)?.val::<bool>() {
                            if *value {
                                if ! set.iter()
                                        .filter(|v|v.mat_style(aopt::prelude::Style::Cmd))
                                        .any(|v|v.matched()) {
                                    cote::simple_display_set_help(set, #name, #head, #foot, #width, #usage_width)
                                        .map_err(|e| aopt::Error::raise_error(format!("Can not display help message: {:?}", e)))?;
                                    std::process::exit(0)
                                }
                            }
                        }
                        Ok(Some(()))
                    }
                );
            });
        }
        for (idx, field) in self.field_metas.iter().enumerate() {
            let ident = Ident::new(&format!("option{}", idx), field.ident.span());
            let config = field.generate_config()?;

            configs.push(quote! {
                let #ident = {
                    ctor.new_with({ #config }).map_err(Into::into)?
                };
            });
            if field.has_handler() {
                let uid_ident = Ident::new(&format!("option_uid_{}", idx), field.ident.span());
                let handler = field.generate_handler(process_help)?;

                inserts.push(quote! {
                    let #uid_ident = set.insert(#ident);
                });
                handlers.push(quote! {
                    parser.entry(#uid_ident)?.on(#handler);
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

    pub fn generate_try_from(&self) -> syn::Result<TokenStream> {
        let generics = &self.struct_meta.generics.params;
        let where_clause = self.struct_meta.generate_where_clause_with_zlifetime()?;
        let ident = self.struct_meta.ident;
        let parser_ty = self.struct_meta.gen_parser_type()?;

        Ok(if generics.is_empty() {
            quote! {
                impl <'zlifetime> std::convert::TryFrom<&'zlifetime mut #parser_ty> for #ident {
                    type Error = aopt::Error;

                    fn try_from(parser: &'zlifetime mut #parser_ty) -> Result<Self, Self::Error> {
                        <#ident as cote::ExtractFromSetDerive<aopt::prelude::ASet>>::try_extract(parser.optset_mut())
                    }
                }
            }
        } else {
            quote! {
                impl <'zlifetime, #generics> std::convert::TryFrom<&'zlifetime mut #parser_ty>
                    for #ident<#generics> where #where_clause {
                    type Error = aopt::Error;

                    fn try_from(parser: &'zlifetime mut #parser_ty) -> Result<Self, Self::Error> {
                        <#ident as cote::ExtractFromSetDerive<aopt::prelude::ASet>>::try_extract(parser.optset_mut())
                    }
                }
            }
        })
    }

    pub fn generate_try_extract(&self) -> syn::Result<TokenStream> {
        let mut mut_field = quote! {};
        let mut ref_field = quote! {};
        let generics = &self.struct_meta.generics.params;
        let where_clause = self.struct_meta.generate_where_clause_with_zlifetime()?;
        let ident = self.struct_meta.ident;

        for field in self.field_metas.iter() {
            let (is_reference, code) = if field.is_sub_command() {
                field.generate_command_extract()?
            } else {
                field.generate_option_extract()?
            };

            if is_reference {
                ref_field.extend(code);
            } else {
                mut_field.extend(code);
            }
        }
        Ok(if generics.is_empty() {
            quote! {
                impl <'zlifetime, S> cote::ExtractFromSetDerive<'zlifetime, S>
                    for #ident where S: aopt::prelude::SetValueFindExt, #where_clause {
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
                impl <'zlifetime, #generics, S> cote::ExtractFromSetDerive<'zlifetime, S>
                    for #ident<#generics> where S: aopt::prelude::SetValueFindExt, #where_clause {
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

    #[allow(unused)]
    tys: Vec<&'a Ident>,

    lifetimes: Vec<&'a Ident>,

    where_clause: Option<&'a Punctuated<WherePredicate, Token!(,)>>,

    global_cfg: Configurations<GlobalCfg>,

    parser_ty: TokenStream,
}

impl<'a> StructMeta<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let params = &generics.params;
        let where_clause = generics.where_clause.as_ref().map(|v| &v.predicates);
        let mut lifetimes = vec![];
        let mut tys = vec![];
        let global_cfg = Configurations::<GlobalCfg>::parse_attrs(&input.attrs, "cote");
        let policy = global_cfg.find_cfg(CfgKind::ParserPolicy);
        let policy_name = policy
            .map(|v| v.value.to_token_stream().to_string())
            .unwrap_or(String::from("fwd"));
        let parser_ty = match policy_name.as_str() {
            "pre" => {
                quote! {
                    aopt::prelude::APreParser<'_>
                }
            }
            "fwd" => {
                quote! {
                    aopt::prelude::AFwdParser<'_>
                }
            }
            "delay" => {
                quote! {
                    aopt::prelude::ADelayParser<'_>
                }
            }
            _ => policy_name.to_token_stream(),
        };

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
            parser_ty,
            where_clause,
        })
    }

    pub fn has_handler(&self) -> bool {
        self.global_cfg.find_cfg(CfgKind::ParserOn).is_some()
            || self.global_cfg.find_cfg(CfgKind::ParserHelp).is_some()
    }

    pub fn generate_where_clause_with_zlifetime(&self) -> syn::Result<TokenStream> {
        let mut code = quote! {};
        let zlifetime = Lifetime::new("'zlifetime", self.ident.span());

        for lifetime in self.lifetimes.iter() {
            let lifetime = Lifetime::new(&format!("'{}", lifetime), lifetime.span());

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

    pub fn generate_where_clause(&self) -> syn::Result<TokenStream> {
        Ok(if let Some(where_clause) = self.where_clause {
            quote! { #where_clause }
        } else {
            quote! {}
        })
    }

    pub fn gen_parser_type(&self) -> syn::Result<&TokenStream> {
        Ok(&self.parser_ty)
    }
}

#[derive(Debug)]
pub struct FieldMeta<'a> {
    ident: Option<&'a Ident>,

    ty: &'a Type,

    trimed_ty: Type,

    unwrap_ty: Option<Type>,

    is_reference: bool,

    field_cfg: Configurations<FieldCfg>,

    is_sub_command: bool,

    is_position: bool,

    parser_ty: Option<TokenStream>,

    comment_doc: Vec<Lit>,
}

impl<'a> FieldMeta<'a> {
    pub fn new(field: &'a Field) -> syn::Result<Self> {
        let ident = field.ident.as_ref();
        let ty = &field.ty;
        let (is_reference, trimed_ty) = remove_lifetime(ty);
        let mut unwrap_ty = None;
        let arg_cfg = Configurations::<ArgCfg>::parse_attrs(&field.attrs, "arg");
        let sub_cfg = Configurations::<SubCfg>::parse_attrs(&field.attrs, "sub");
        let comment_doc = filter_comment_doc(&field.attrs);
        let is_sub_command;
        let is_position = check_in_path(ty, "Pos") || check_in_path(ty, "Cmd");
        let mut parser_ty = None;

        let field_cfg = if !arg_cfg.cfgs.is_empty() && !sub_cfg.cfgs.is_empty() {
            abort! {
                ident,
                "can not have both `arg` and `sub` on one field",
            }
        } else if !arg_cfg.cfgs.is_empty() {
            is_sub_command = false;
            Configurations {
                cfgs: arg_cfg.cfgs.into_iter().map(|v| v.into()).collect(),
            }
        } else if !sub_cfg.cfgs.is_empty() {
            let policy = sub_cfg.find_cfg(CfgKind::SubPolicy);
            let policy_name = policy
                .map(|v| v.value.to_token_stream().to_string())
                .unwrap_or(String::from("fwd"));

            parser_ty = Some(match policy_name.as_str() {
                "pre" => {
                    quote! {
                        aopt::prelude::APreParser<'_>
                    }
                }
                "fwd" => {
                    quote! {
                        aopt::prelude::AFwdParser<'_>
                    }
                }
                "delay" => {
                    quote! {
                        aopt::prelude::ADelayParser<'_>
                    }
                }
                _ => policy_name.to_token_stream(),
            });
            unwrap_ty = Some(remove_option(&trimed_ty)?);

            is_sub_command = true;
            Configurations {
                cfgs: sub_cfg.cfgs.into_iter().map(|v| v.into()).collect(),
            }
        } else {
            is_sub_command = false;
            Configurations { cfgs: vec![] }
        };
        Ok(Self {
            ident,
            ty,
            trimed_ty,
            field_cfg,
            parser_ty,
            unwrap_ty,
            comment_doc,
            is_reference,
            is_position,
            is_sub_command,
        })
    }

    pub fn generate_handler(&self, process_help: bool) -> syn::Result<TokenStream> {
        if let Some(cfg) = self.field_cfg.find_cfg(CfgKind::OptOn) {
            let value = &cfg.value;

            Ok(quote! {
                #value
            })
        } else if self.is_sub_command() {
            let parser_ty = self.parser_ty.as_ref().unwrap();
            let unwrap_ty = self.unwrap_ty.as_ref().unwrap();
            let pass_help_to_next = if process_help {
                let uid_ident = Ident::new(HELP_OPTION_UID, self.ident.span());

                quote! {
                    if let Ok(value) = set.opt(#uid_ident)?.val::<bool>() {
                        if *value {
                            // pass a fake flag to next sub command
                            args.push(aopt::RawVal::from("--help"));
                        }
                    }
                }
            } else {
                quote! {}
            };

            Ok(quote! {
                move |set: &mut P::Set, _: &mut P::Ser, args: aopt::prelude::ctx::Args, index: aopt::prelude::ctx::Index| {
                    use std::ops::Deref;

                    let mut args = args.deref().clone().into_inner();

                    // remove current sub command
                    args.remove(*index.deref());
                    #pass_help_to_next

                    let args = aopt::ARef::new(aopt::prelude::Args::from_vec(args));
                    let dbg_args = args.clone();
                    let mut parser: #parser_ty = <#unwrap_ty>::into_parser()?;

                    parser.init()?;
                    match parser.parse(args).map_err(Into::into)?.ok() {
                        Ok(_) => {
                            Ok(<#unwrap_ty>::try_extract(parser.optset_mut()).ok())
                        }
                        Err(e) => {
                            Err(aopt::Error::raise_error(
                                format!("parsing arguments failed! {{parser: {}, args: {:?}}}: {:?}",
                                    stringify!(#unwrap_ty),
                                    dbg_args, e)))
                        }
                    }
                }
            })
        } else {
            unreachable!("can not generate handler for field: {:?}", self.ident)
        }
    }

    pub fn generate_field_name(&self) -> String {
        let ident = self
            .ident
            .unwrap_or_else(|| {
                abort! {
                    self.ident,
                    "missing filed name",
                }
            })
            .to_string();

        if self.is_position || self.is_sub_command() {
            ident
        } else if ident.chars().count() >= 2 {
            format!("--{}", ident)
        } else {
            format!("-{}", ident)
        }
    }

    pub fn has_handler(&self) -> bool {
        self.field_cfg.find_cfg(CfgKind::OptOn).is_some() || self.is_sub_command()
    }

    pub fn is_sub_command(&self) -> bool {
        self.is_sub_command
    }

    pub fn generate_command_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let is_ref = self.field_cfg.find_cfg(CfgKind::SubRef).is_some();
        let is_mut = self.field_cfg.find_cfg(CfgKind::SubMut).is_some();
        let ident = self.ident;
        let name = self.generate_field_name().to_token_stream();
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
                    #ident: set.find_val(#name).ok(),
                },
            ))
        } else if is_mut {
            Ok((
                false,
                quote! {
                    #ident: set.take_val(#name).ok(),
                },
            ))
        } else if self.is_reference {
            Ok((
                true,
                quote! {
                    #ident:  set.find_val(#name).ok(),
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: set.take_val(#name).ok(),
                },
            ))
        }
    }

    pub fn generate_option_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let is_ref = self.field_cfg.find_cfg(CfgKind::OptRef).is_some();
        let is_mut = self.field_cfg.find_cfg(CfgKind::OptMut).is_some();
        let ident = self.ident;
        let name = self.generate_field_name().to_token_stream();
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
                    #ident: aopt::prelude::InferValueRef::infer_fetch(#name, set)?,
                },
            ))
        } else if is_mut {
            Ok((
                false,
                quote! {
                    #ident: aopt::prelude::InferValueMut::infer_fetch(#name, set)?,
                },
            ))
        } else if self.is_reference {
            Ok((
                true,
                quote! {
                    #ident: aopt::prelude::InferValueRef::infer_fetch(#name, set)?,
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: aopt::prelude::InferValueMut::infer_fetch(#name, set)?,
                },
            ))
        }
    }

    pub fn generate_config(&self) -> syn::Result<TokenStream> {
        if self.is_sub_command() {
            self.generate_command_config()
        } else {
            self.generate_option_config()
        }
    }

    pub fn generate_command_config(&self) -> syn::Result<TokenStream> {
        let config = &self.field_cfg;
        let ident = self.ident;
        let mut ret = quote! {
            let mut config = aopt::prelude::SetCfg::<P::Set>::default();
        };
        let mut has_name = false;
        let mut codes = vec![];

        for cfg in config.cfgs.iter() {
            codes.push(match cfg.kind {
                CfgKind::SubPolicy => {
                    quote! {}
                }
                CfgKind::SubName => {
                    let token = cfg.value.to_token_stream();

                    has_name = true;
                    quote! {
                        config.set_name(#token);
                    }
                }
                CfgKind::SubAlias => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.add_alias(#token);
                    }
                }
                CfgKind::SubHint => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_hint(#token);
                    }
                }
                CfgKind::SubHelp => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_help(#token);
                    }
                }
                _ => {
                    abort! {
                        ident, "Unsupport config kind on field macro `sub`: {:?}", cfg.kind
                    }
                }
            });
        }

        if !has_name {
            let ident = self.generate_field_name();

            codes.push(quote! {
                config.set_name(#ident);
            });
        }
        if self.field_cfg.find_cfg(CfgKind::SubHelp).is_none() && !self.comment_doc.is_empty() {
            let mut code = quote! {
                let mut message = String::default();
            };
            for doc in self.comment_doc.iter() {
                code.extend(quote! {
                    message.push_str(#doc);
                });
            }
            codes.push(quote! {
                config.set_help({ #code message });
            })
        }
        codes.push(quote! {
            aopt::opt::Cmd::infer_fill_info(&mut config, true);
            config
        });
        ret.extend(codes.into_iter());
        Ok(ret)
    }

    pub fn generate_option_config(&self) -> syn::Result<TokenStream> {
        let ty = self.ty;
        let ident = self.ident;
        let config = &self.field_cfg;
        let trimed_ty = &self.trimed_ty;
        let mut value = None;

        let mut codes = vec![];
        let mut has_name = false;
        let mut ret = quote! {
            let mut config = aopt::prelude::SetCfg::<P::Set>::default();
        };

        for cfg in config.cfgs.iter() {
            codes.push(match cfg.kind {
                CfgKind::OptHint => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_hint(#token);
                    }
                }
                CfgKind::OptName => {
                    let token = cfg.value.to_token_stream();

                    has_name = true;
                    quote! {
                        config.set_name(#token);
                    }
                }
                CfgKind::OptForce => {
                    quote! {
                        config.set_name(true);
                    }
                }
                CfgKind::OptNoForce => {
                    quote! {
                        config.set_name(false);
                    }
                }
                CfgKind::OptValue => {
                    let token = cfg.value.to_token_stream();

                    value = Some(token.clone());
                    quote! {
                        config.set_initializer(aopt::prelude::ValInitializer::new_value(<<#ty as aopt::prelude::Infer>::Val>::from(#token)));
                    }
                }
                CfgKind::OptValues => {
                    let token = cfg.value.to_token_stream();

                    value = Some(token.clone());
                    quote! {
                        let values = #token.into_iter().map(|v|<<#ty as aopt::prelude::Infer>::Val>::from(v)).collect::<Vec<<#ty as aopt::prelude::Infer>::Val>>();
                        config.set_initializer(aopt::prelude::ValInitializer::new_values(values));
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
                        config.set_index(aopt::prelude::Index::parse(#token)?);
                    }
                }
                CfgKind::OptValidator => {
                    let token = cfg.value.to_token_stream();

                    quote! {
                        config.set_storer(aopt::prelude::ValStorer::new_validator::<#ty>(#token));
                    }
                }
                CfgKind::OptOn | CfgKind::OptRef | CfgKind::OptMut | CfgKind::OptHelp => {
                    // will process in another function
                    quote! { }
                }
                _ => {
                    abort! {
                        ident, "Unsupport config kind on field macro `arg`: {:?}", cfg.kind
                    }
                }
            });
        }
        if !has_name {
            let name = self.generate_field_name();

            codes.push(quote! {
                config.set_name(#name);
            });
        }
        let mut help_code = None;

        if let Some(help_cfg) = self.field_cfg.find_cfg(CfgKind::OptHelp) {
            let token = &help_cfg.value;

            help_code = Some(quote! {
                let mut message = String::from(#token.trim());

                message.push_str(" ");
            });
        } else if !self.comment_doc.is_empty() {
            help_code = Some({
                let mut code = quote! {
                    let mut message = String::default();
                };
                for doc in self.comment_doc.iter() {
                    code.extend(quote! {
                        message.push_str(#doc.trim());
                        message.push_str(" ");
                    });
                }
                code
            });
        }
        if let Some(mut help_code) = help_code {
            if let Some(value) = value {
                help_code.extend(quote! {
                    message.push_str("[");
                    message.push_str(#value.trim());
                    message.push_str("]");
                });
            }
            codes.push(quote! {
                config.set_help({ #help_code message });
            })
        }
        codes.push(quote! {
            <#trimed_ty>::infer_fill_info(&mut config, true);
            config
        });
        ret.extend(codes.into_iter());
        Ok(ret)
    }
}

pub fn remove_option(ty: &Type) -> syn::Result<Type> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            let ident = segment.ident.to_string();

            if ident == "Option" {
                match &segment.arguments {
                    PathArguments::AngleBracketed(ab) => {
                        if let Some(GenericArgument::Type(next_ty)) = ab.args.first().as_ref() {
                            return Ok(next_ty.clone());
                        } else {
                            abort! {
                                ty,
                                "`sub` not support current type"
                            }
                        }
                    }
                    _ => {
                        abort! {
                            ty,
                            "`sub` not support current type"
                        }
                    }
                }
            } else {
                abort! {
                    ty,
                    "`sub` must wrapped with Option"
                }
            }
        }
    }
    Ok(ty.clone())
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
                if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                    for arg in ab.args.iter() {
                        if let GenericArgument::Type(next_ty) = arg {
                            return check_if_reference(next_ty);
                        }
                    }
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

pub fn check_in_path(ty: &Type, name: &str) -> bool {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            let ident = segment.ident.to_string();

            if ident == name {
                return true;
            } else if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                for arg in ab.args.iter() {
                    if let GenericArgument::Type(next_ty) = arg {
                        return check_in_path(next_ty, name);
                    }
                }
            }
        }
    } else if let Type::Reference(reference) = ty {
        return check_in_path(reference.elem.as_ref(), name);
    }
    false
}
