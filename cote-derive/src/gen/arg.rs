use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Field;
use syn::GenericArgument;
use syn::Lit;
use syn::PathArguments;
use syn::Type;

use crate::config;
use crate::config::ArgKind;
use crate::config::Configs;
use crate::error;

use super::check_in_path;
use super::filter_comment_doc;
use super::gen_option_ident;
use super::gen_option_uid_ident;
use super::OptUpdate;
use super::CONFIG_ARG;
use super::CONFIG_CMD;
use super::CONFIG_POS;

#[derive(Debug)]
pub struct ArgGenerator<'a> {
    field_ty: &'a Type,

    name: TokenStream,

    ident: Option<&'a Ident>,

    docs: Vec<Lit>,

    configs: Configs<ArgKind>,

    pos_id: Option<usize>,

    cfg_name: &'static str,

    type_hint: TypeHint<'a>,
}

impl<'a> ArgGenerator<'a> {
    pub fn new(field: &'a Field, pos_id: usize) -> syn::Result<Self> {
        let field_ty = &field.ty;
        let ident = field.ident.as_ref();
        let attrs = &field.attrs;
        let docs = filter_comment_doc(attrs);
        let cfg_name = config::find_cfg_name(&[CONFIG_ARG, CONFIG_POS, CONFIG_CMD], attrs)
            .unwrap_or(CONFIG_ARG);
        let configs = Configs::parse_attrs(cfg_name, attrs);
        let type_hint = TypeHint::new(field_ty);
        let is_pos_ty = check_in_path(field_ty, "Pos")?;
        let is_cmd_ty = check_in_path(field_ty, "Cmd")?;
        let is_main_ty = check_in_path(field_ty, "Main")?;
        let has_index = configs.has_cfg(ArgKind::Index);
        let is_position = is_pos_ty
            || is_cmd_ty
            || is_main_ty
            || has_index
            || cfg_name == CONFIG_POS
            || cfg_name == CONFIG_CMD;
        let pos_id = if (is_pos_ty || cfg_name == CONFIG_POS) && !has_index {
            Some(pos_id)
        } else {
            None
        };
        let name = {
            if let Some(cfg) = configs.find_cfg(ArgKind::Name) {
                cfg.value().to_token_stream()
            } else {
                if ident.is_none() {
                    return error(
                        field.span(),
                        "`arg`, `pos` or `cmd` not support empty field name".to_owned(),
                    );
                }
                let ident = ident.unwrap();
                let ident = ident.to_string();
                let name = if is_position {
                    ident
                } else if ident.chars().count() >= 2 {
                    format!("--{}", ident.replace('_', "-"))
                } else {
                    format!("-{}", ident)
                };

                quote! { #name }
            }
        };

        if (cfg_name == CONFIG_CMD || is_cmd_ty || is_main_ty) && has_index {
            return error(
                field_ty.span(),
                "`cmd` has default position, please remove the `index` attribute".to_owned(),
            );
        }
        if configs.has_cfg(ArgKind::Action)
            && (configs.has_cfg(ArgKind::Append) || configs.has_cfg(ArgKind::Count))
        {
            return error(
                field_ty.span(),
                "`app` and `cnt` are alias of `action`, try to remove one from the configures"
                    .to_owned(),
            );
        }
        Ok(Self {
            name,
            ident,
            docs,
            configs,
            pos_id,
            field_ty,
            cfg_name,
            type_hint,
        })
    }

    pub fn has_pos_id(&self) -> bool {
        self.pos_id.is_some()
    }

    pub fn has_handler(&self) -> bool {
        self.configs.has_cfg(ArgKind::On)
            || self.configs.has_cfg(ArgKind::Then)
            || self.configs.has_cfg(ArgKind::Fallback)
    }

    pub fn gen_nodelay_for_delay_parser(&self) -> Option<TokenStream> {
        self.configs.find_cfg(ArgKind::NoDelay).map(|_| {
            let name = &self.name;

            quote! {
                cote::PolicySettings::set_no_delay(policy, #name);
            }
        })
    }

    pub fn gen_value_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let ident = self.ident;
        let name = &self.name;
        let hint = self.type_hint;
        let fetch = self.configs.find_cfg(ArgKind::Fetch);
        let inner_ty = self.type_hint.inner_type();
        let spec_ty = self.configs.find_cfg(ArgKind::Type);

        if let Some(fetch) = fetch {
            let func = fetch.value();

            if let Some(spec_ty) = spec_ty {
                let inner_ty = spec_ty.value();

                Ok((
                    false,
                    quote! {
                        #ident: #func::<#inner_ty>(#name, set)?,
                    },
                ))
            } else {
                Ok((
                    false,
                    quote! {
                        #ident: #func::<#inner_ty>(#name, set)?,
                    },
                ))
            }
        } else {
            match hint {
                TypeHint::Opt(_) => Ok((
                    false,
                    quote! {
                        #ident: cote::Fetch::fetch(#name, set).ok(),
                    },
                )),
                TypeHint::Vec(_) => Ok((
                    false,
                    quote! {
                        #ident: cote::Fetch::fetch_vec(#name, set)?,
                    },
                )),
                TypeHint::OptVec(_) => Ok((
                    false,
                    quote! {
                        #ident: cote::Fetch::fetch_vec(#name, set).ok(),
                    },
                )),
                TypeHint::Null(_) => Ok((
                    false,
                    quote! {
                        #ident: cote::Fetch::fetch(#name, set)?,
                    },
                )),
            }
        }
    }

    pub fn gen_option_update(&self, idx: usize) -> syn::Result<OptUpdate> {
        let ident = gen_option_ident(idx, self.ident.span());
        let uid = gen_option_uid_ident(idx, self.ident.span());

        Ok((
            Some(self.gen_option_config_new(&ident)?),
            Some(self.gen_option_config_insert(&uid, &ident)),
            self.gen_option_handler_insert(&uid)?,
        ))
    }

    pub fn gen_option_handler_insert(&self, uid: &Ident) -> syn::Result<Option<TokenStream>> {
        if let Some(cfg) = self.configs.find_cfg(ArgKind::On) {
            let value = cfg.value();

            Ok(Some(
                if let Some(then_cfg) = self.configs.find_cfg(ArgKind::Then) {
                    let then = then_cfg.value();

                    quote! {
                        parser.entry(#uid)?.on(#value).then(#then);
                    }
                } else {
                    quote! {
                        parser.entry(#uid)?.on(#value);
                    }
                },
            ))
        } else if let Some(cfg) = self.configs.find_cfg(ArgKind::Fallback) {
            let value = cfg.value();

            Ok(Some(
                if let Some(fallback) = self.configs.find_cfg(ArgKind::Then) {
                    let then = fallback.value();

                    quote! {
                        parser.entry(#uid)?.fallback(#value).then(#then);
                    }
                } else {
                    quote! {
                        parser.entry(#uid)?.fallback(#value);
                    }
                },
            ))
        } else if self.configs.has_cfg(ArgKind::Then) {
            return error(
                self.field_ty.span(),
                "`then` must use with `on` or `fallback` together".to_owned(),
            );
        } else {
            Ok(None)
        }
    }

    pub fn gen_option_config_insert(&self, uid: &Ident, ident: &Ident) -> TokenStream {
        if self.has_handler() {
            quote! {
                let #uid = set.insert(#ident);
            }
        } else {
            quote! {
                set.insert(#ident);
            }
        }
    }

    pub fn gen_option_config_new(&self, ident: &Ident) -> syn::Result<TokenStream> {
        let type_hint = self.type_hint;
        let inner_ty = type_hint.inner_type();
        let name = &self.name;
        let mut codes = vec![];
        let mut value = None;
        let mut config = quote! {
            let mut config = cote::SetCfg::<Set>::default();
            config.set_name(#name);
        };

        // generate option create
        for cfg in self.configs.iter() {
            codes.push(
                    match cfg.kind() {
                        ArgKind::Hint => {
                            let token = cfg.value();

                            quote! {
                                config.set_hint(#token);
                            }
                        }
                        ArgKind::Value => {
                            let token = cfg.value();

                            value = Some(token.clone());
                            quote! {
                                config.set_initializer(
                                    cote::ValInitializer::new_value(
                                        <<#inner_ty as cote::Infer>::Val>::from(#token)
                                    )
                                );
                            }
                        }
                        ArgKind::Values => {
                            let token = cfg.value();

                            value = Some(token.clone());
                            quote! {
                                let values = #token.into_iter().map(
                                    |v|<<#inner_ty as cote::Infer>::Val>::from(v)
                                ).collect::<Vec<<#inner_ty as cote::Infer>::Val>>();
                                config.set_initializer(cote::ValInitializer::new_values(values));
                            }
                        }
                        ArgKind::Alias => {
                            let token = cfg.value();

                            quote! {
                                config.add_alias(#token);
                            }
                        }
                        ArgKind::Index => {
                            let token = cfg.value();

                            quote! {
                                config.set_index(<cote::Index as std::convert::TryFrom::<_>>::try_from(#token)?);
                            }
                        }
                        ArgKind::Force => {
                            let token = cfg.value();

                            quote! {
                                config.set_force(#token);
                            }
                        }
                        ArgKind::Action => {
                            let token = cfg.value();

                            quote! {
                                config.set_action(#token);
                            }
                        }
                        ArgKind::Validator => {
                            let token = cfg.value();

                            quote! {
                                let validator = cote::ValValidator::from_fn(|value| {
                                    use cote::valid::Validate;
                                    #token.check(value)
                                });
                                config.set_storer(
                                    cote::ValStorer::new_validator::<<#inner_ty as cote::Infer>::Val>(validator)
                                );
                            }
                        }
                        ArgKind::MethodCall(method) => {
                            let method = Ident::new(method, inner_ty.span());
                            let value = cfg.value().clone();
                            let (var, args) = value.split_call_args(inner_ty.span())?;
                            let var_name = var.to_token_stream().to_string();

                            match var_name.as_str() {
                                "config" => {
                                    quote!{
                                        #var.#method(#args);
                                    }
                                }
                                _ => {
                                    let args = cfg.value();

                                    quote!{
                                        #method(#args);
                                    }
                                }
                            }
                        }
                        ArgKind::Count => {
                            quote! {
                                config.set_action(cote::Action::Cnt);
                            }
                        }
                        ArgKind::Append => {
                            quote! {
                                config.set_action(cote::Action::App);
                            }
                        }
                        _ => {
                            quote!{}
                        }
                    }
               )
        }
        if self.configs.has_cfg(ArgKind::Value) || self.configs.has_cfg(ArgKind::Values) {
            codes.push(quote! {
                config.set_force(false);
            })
        }
        let help = if let Some(cfg) = self.configs.find_cfg(ArgKind::Help) {
            let value = cfg.value();
            Some(quote! { let mut message = String::from(#value.trim()); })
        } else if !self.docs.is_empty() {
            Some({
                let mut code = quote! {
                    let mut message = String::default();
                };
                let mut iter = self.docs.iter();

                if let Some(doc) = iter.next() {
                    code.extend(quote! {
                        message.push_str(#doc.trim());
                    });
                }
                for doc in iter {
                    code.extend(quote! {
                        message.push_str(" ");
                        message.push_str(#doc.trim());
                    });
                }
                code
            })
        } else {
            None
        };

        if let Some(mut help) = help {
            if let Some(value) = &value {
                let value_string = value.to_token_stream().to_string();

                help.extend(quote! {
                    message.push_str(" ");
                    message.push_str("[");
                    message.push_str(#value_string.trim());
                    message.push_str("]");
                });
            }
            codes.push(quote! {
                config.set_help({ #help message });
            })
        }
        if let Some(pos_id) = self.pos_id {
            if !self.configs.has_cfg(ArgKind::Index) {
                codes.push(quote! {
                    config.set_index(cote::Index::forward(#pos_id));
                })
            } else {
                return error(
                    inner_ty.span(),
                    format!(
                        "Can not have both auto increment Pos id and index configuration `{:?}`",
                        self.configs.find_cfg(ArgKind::Index)
                    ),
                );
            }
        }
        if let Some(cfg) = self.configs.find_cfg(ArgKind::Type) {
            let spec_ty = cfg.value();

            codes.push(quote! {
                <#spec_ty as cote::Alter>::alter(cote::Hint::Null, &mut config);
                <#spec_ty as cote::Infer>::infer_fill_info(&mut config, true);
            });
        } else {
            match self.cfg_name {
                CONFIG_CMD => {
                    codes.push(if !type_hint.is_null() {
                        return error(
                            self.field_ty.span(),
                            "Cmd always force required, please remove Option or Vec from type"
                                .to_owned(),
                        );
                    } else {
                        quote! {
                            config.set_type::<#inner_ty>();
                            <cote::Cmd as cote::Alter>::alter(cote::Hint::Null, &mut config);
                            <cote::Cmd as cote::Infer>::infer_fill_info(&mut config, true);
                        }
                    });
                }
                CONFIG_POS => {
                    codes.push(match type_hint {
                        TypeHint::Opt(inner_ty) => {
                            quote! {
                                // using information of Pos<T>
                                config.set_type::<#inner_ty>();
                                <cote::Pos<#inner_ty> as cote::Alter>::alter(cote::Hint::Opt, &mut config);
                                <cote::Pos<#inner_ty> as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        },
                        TypeHint::Vec(inner_ty) => {
                            quote! {
                                // using information of Pos<T>
                                config.set_type::<#inner_ty>();
                                <cote::Pos<#inner_ty> as cote::Alter>::alter(cote::Hint::Vec, &mut config);
                                <cote::Pos<#inner_ty> as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        },
                        TypeHint::OptVec(inner_ty) => {
                            quote! {
                                // using information of Pos<T>
                                config.set_type::<#inner_ty>();
                                <cote::Pos<#inner_ty> as cote::Alter>::alter(cote::Hint::OptVec, &mut config);
                                <cote::Pos<#inner_ty> as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        },
                        TypeHint::Null(inner_ty) => {
                            quote! {
                                // using information of Pos<T>
                                config.set_type::<#inner_ty>();
                                <cote::Pos<#inner_ty> as cote::Alter>::alter(cote::Hint::Null, &mut config);
                                <cote::Pos<#inner_ty> as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        },
                    });
                }
                _ => {
                    codes.push(match type_hint {
                        TypeHint::Opt(inner_ty) => {
                            quote! {
                                <#inner_ty as cote::Alter>::alter(cote::Hint::Opt, &mut config);
                                <#inner_ty as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        }
                        TypeHint::Vec(inner_ty) => {
                            quote! {
                                <#inner_ty as cote::Alter>::alter(cote::Hint::Vec, &mut config);
                                <#inner_ty as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        }
                        TypeHint::OptVec(inner_ty) => {
                            quote! {
                                <#inner_ty as cote::Alter>::alter(cote::Hint::OptVec, &mut config);
                                <#inner_ty as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        }
                        TypeHint::Null(inner_ty) => {
                            quote! {
                                <#inner_ty as cote::Alter>::alter(cote::Hint::Null, &mut config);
                                <#inner_ty as cote::Infer>::infer_fill_info(&mut config, true);
                            }
                        }
                    });
                }
            }
        }

        config.extend(codes.into_iter());
        config.extend(quote! { config });

        Ok(quote! {
            let #ident = {
                ctor.new_with({ #config }).map_err(Into::into)?
            };
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TypeHint<'a> {
    Opt(&'a Type),

    Vec(&'a Type),

    OptVec(&'a Type),

    Null(&'a Type),
}

impl<'a> TypeHint<'a> {
    pub fn new(ty: &'a Type) -> Self {
        match check_segment_ty(ty, "Option") {
            (true, inner_ty) => match check_segment_ty(inner_ty, "Vec") {
                (true, inner_ty) => Self::OptVec(inner_ty),
                (false, inner_ty) => Self::Opt(inner_ty),
            },
            (false, inner_ty) => match check_segment_ty(inner_ty, "Vec") {
                (true, inner_ty) => Self::Vec(inner_ty),
                (false, _) => Self::Null(ty),
            },
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null(_))
    }

    pub fn inner_type(&self) -> &Type {
        match self {
            TypeHint::Opt(ty) => ty,
            TypeHint::Vec(ty) => ty,
            TypeHint::OptVec(ty) => ty,
            TypeHint::Null(ty) => ty,
        }
    }
}

pub fn check_segment_ty<'a>(ty: &'a Type, name: &str) -> (bool, &'a Type) {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            let ident_str = segment.ident.to_string();

            if ident_str == name {
                if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                    if let Some(GenericArgument::Type(next_ty)) = ab.args.first().as_ref() {
                        return (true, next_ty);
                    }
                }
            }
        }
    }
    (false, ty)
}
