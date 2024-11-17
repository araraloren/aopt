use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Field, Ident, Type};

use crate::{config::ArgKind, error};

use super::{AttrKind, Utils};
use super::{FieldCfg, OptUpdate};

#[derive(Debug)]
pub struct ArgGenerator<'a> {
    name: TokenStream,

    ident: Ident,

    uid_ident: Ident,

    index: Option<usize>,

    config: FieldCfg<'a, ArgKind>,
}

impl<'a> ArgGenerator<'a> {
    pub fn new(field: &'a Field, id: u64, kind: AttrKind) -> syn::Result<Self> {
        let config = FieldCfg::new(id, field, kind)?;
        let index = config.has_cfg(ArgKind::Index);
        let ident = Utils::id2opt_ident(id, field.span());
        let uid_ident = Utils::id2opt_uid_ident(id, field.span());
        let name = config
            .find_value(ArgKind::Name)
            .map(|v| v.to_token_stream())
            .unwrap_or_else(|| {
                let ident_str = config.ident().to_string();

                if kind.is_cmd() || kind.is_pos() || kind.is_main() || index {
                    ident_str.to_token_stream()
                } else {
                    Utils::ident2opt_name(&ident_str).to_token_stream()
                }
            });

        if (kind.is_cmd() || kind.is_main()) && config.has_cfg(ArgKind::Index) {
            Err(error(
                field.span(),
                format!(
                    "`cmd` has default index, please remove the `index` attribute from `{:?}`",
                    config.ident()
                ),
            ))
        } else if config.has_cfg(ArgKind::Action)
            && (config.has_cfg(ArgKind::Append) || config.has_cfg(ArgKind::Count))
        {
            Err(error(
                field.span(),
                "`app` and `cnt` are alias of `action`, please remove one from attributes",
            ))
        } else {
            Ok(Self {
                name,
                index: None,
                config,
                ident,
                uid_ident,
            })
        }
    }

    pub fn uid(&self) -> u64 {
        self.config.id()
    }

    pub fn kind(&self) -> AttrKind {
        self.config.kind()
    }

    pub fn ty(&self) -> &'a Type {
        self.config.ty()
    }

    pub fn orig_ident(&self) -> &'a Ident {
        self.config.ident()
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn uid_ident(&self) -> &Ident {
        &self.uid_ident
    }

    pub fn need_pos_index(&self) -> bool {
        self.kind().is_pos() && !self.config.has_cfg(ArgKind::Index)
    }

    // index using for generate POS
    pub fn set_pos_index(&mut self, index: usize) -> &mut Self {
        self.index = Some(index);
        self
    }

    pub fn pos_index(&self) -> Option<usize> {
        self.index
    }

    pub fn gen_opt_update(&self) -> syn::Result<OptUpdate> {
        let c = self.gen_opt_create()?;
        let i = self.gen_opt_insert()?;
        let h = self.gen_opt_handler()?;

        Ok(OptUpdate {
            h,
            ..Default::default()
        }
        .with_create(c)
        .with_insert(i))
    }

    pub fn gen_opt_insert(&self) -> syn::Result<TokenStream> {
        let ident = self.ident();
        let uid_ident = self.uid_ident();
        let uid_literal = Utils::id2uid_literal(self.uid());

        Utils::gen_opt_insert(ident, uid_ident, &uid_literal)
    }

    pub fn gen_opt_handler(&self) -> syn::Result<Option<TokenStream>> {
        let on = self.config.find_cfg(ArgKind::On);
        let fallback = self.config.find_cfg(ArgKind::Fallback);
        let then = self.config.find_cfg(ArgKind::Then);
        let uid_ident = self.uid_ident();

        Utils::gen_opt_handler(uid_ident, on, fallback, then)
    }

    pub fn gen_opt_create(&self) -> syn::Result<TokenStream> {
        let field_span = self.ident().span();
        let field_ty = self.ty();
        let field_cfg = &self.config;
        let cfg_ident = Ident::new("cfg", field_span);
        let mut codes = vec![];
        let mut value = None;

        codes.push(ArgKind::Name.simple(&cfg_ident, self.name.clone())?);
        for cfg in field_cfg.configs().iter() {
            let kind = cfg.kind();
            let cfg_value = cfg.value();

            match kind {
                ArgKind::Hint
                | ArgKind::Alias
                | ArgKind::Force
                | ArgKind::Action
                | ArgKind::Count
                | ArgKind::Index
                | ArgKind::Append => {
                    let value = cfg_value.to_token_stream();

                    codes.push(kind.simple(&cfg_ident, value)?);
                }
                ArgKind::Name => {
                    // already processed
                }
                ArgKind::Type | ArgKind::Help => {
                    // process later
                }
                ArgKind::On | ArgKind::Fallback | ArgKind::Then => {}

                ArgKind::Validator => codes.push(kind.simple(
                    &cfg_ident,
                    quote! {{
                        let validator = cote::prelude::ValValidator::from_fn( |value| {
                            cote::valid::Validate::check(& #cfg_value, value)
                        });
                        cote::prelude::ValStorer::new_validator::<InferedOptVal<#field_ty>>(validator)
                    }},
                )?),
                ArgKind::MethodCall(method) => {
                    let method = Ident::new(method.as_str(), field_span);
                    let value = cfg_value.clone();
                    let (_self, args) = value.split_call_args(field_span)?;
                    let caller = _self.to_token_stream().to_string();

                    codes.push(match caller.as_str() {
                        "config" | "cfg" => quote! {
                            #method::<Set>(&mut #cfg_ident, #args);
                        },
                        _ => quote! { #method(#cfg_value); },
                    });
                }
                ArgKind::Value => {
                    value = Some(cfg_value.clone());
                    codes.push(kind.simple(
                        &cfg_ident,
                        quote!( <InferedOptVal<#field_ty>>::from(#cfg_value) ),
                    )?);
                }
                ArgKind::Values => {
                    value = Some(cfg_value.clone());

                    codes.push(kind.simple(
                        &cfg_ident,
                        quote!( #cfg_value.into_iter().map(<InferedOptVal<#field_ty>>::from).collect::<Vec<InferedOptVal<#field_ty>>>()
                        ),
                    )?);
                }
                ArgKind::NoDelay => {
                    // will process in policy settings 
                },
                ArgKind::Fetch => {
                    // will process in try extract
                },
            }
        }
        // if we have value, set the force to false
        if value.is_some() {
            codes.push(ArgKind::Force.simple(&cfg_ident, false.to_token_stream())?);
        }
        if let Some(help) = field_cfg
            .find_value(ArgKind::Help)
            .map(|v| quote! { String::from(#v.trim()) })
            .or_else(|| field_cfg.collect_help_msgs())
        {
            codes.push(ArgKind::Help.simple(
                &cfg_ident,
                if let Some(value) = value.as_ref() {
                    let value = value.to_token_stream();

                    // using Debug for default value, better?
                    quote! { format!("{} [{:?}]", #help, #value) }
                } else {
                    help
                },
            )?);
        }
        if let Some(index) = self.pos_index() {
            if !self.config.has_cfg(ArgKind::Index) {
                codes.push(quote! {
                    cote::prelude::ConfigValue::set_index(&mut #cfg_ident, cote::prelude::Index::forward(#index));
                });
            } else {
                return Err(error(
                    field_span,
                    format!(
                        "Can not have both auto increment index and index attribute on field `{}`",
                        self.ident,
                    ),
                ));
            }
        }
        codes.push(if let Some(ty) = self.config.find_value(ArgKind::Type) {
            quote! {
                <#ty as cote::prelude::Infer>::infer_fill_info(&mut #cfg_ident)?;
            }
        } else {
            self.kind().gen_infer(&cfg_ident, field_ty)?
        });
        Utils::gen_opt_create(self.ident(), Some(quote! { #(#codes)* }))
    }

    pub fn gen_try_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let ident = self.orig_ident();
        let field_ty = self.ty();
        let fetch = self.config.find_cfg(ArgKind::Fetch);
        let uid_literal = Utils::id2uid_literal(self.uid());
        // let spec_ty = self.config.find_cfg(ArgKind::Type);
        // don't use spec_ty here, let user choose how to fetch value

        if let Some(fetch) = fetch {
            let func = fetch.value();

            Ok((
                false,
                quote! {
                    #ident: #func::<#field_ty, Set>(#uid_literal, set)?
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: cote::prelude::Fetch::<Set>::fetch_uid(#uid_literal, set)?
                },
            ))
        }
    }

    pub fn gen_nodelay_setting(&self) -> syn::Result<Option<TokenStream>> {
        let name = &self.name;
        Ok(self.config.has_cfg(ArgKind::NoDelay).then_some({
            quote! {
                cote::prelude::PolicySettings::set_no_delay(policy, #name);
            }
        }))
    }
}
