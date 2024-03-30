use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::DataStruct;
use syn::DeriveInput;
use syn::Field;
use syn::Fields;
use syn::GenericParam;
use syn::Type;
use syn::{Data, Generics};

use crate::config::Configs;
use crate::config::CoteKind;
use crate::error;
use crate::gen::GenericsModifier;

use super::arg::ArgGenerator;
use super::sub::SubGenerator;
use super::AttrKind;
use super::OptUpdate;
use super::Utils;
use super::CONFIG_ARG;
use super::CONFIG_CMD;
use super::CONFIG_POS;
use super::CONFIG_SUB;
use super::POLICY_FWD;
use super::POLICY_PRE;

#[derive(Debug)]
pub struct CoteGenerator<'a> {
    name: TokenStream,

    orig_ident: &'a Ident,

    field_generators: Vec<FieldGenerator<'a>>,

    generics: &'a Generics,

    configs: Configs<CoteKind>,

    help_uid: Option<u64>,

    main_uid: Option<u64>,
}

impl<'a> CoteGenerator<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let orig_ident = &input.ident;
        let generics = &input.generics;
        let configs = Configs::<CoteKind>::parse_attrs("cote", &input.attrs);
        let name = configs
            .find_cfg(CoteKind::Name)
            .map(|v| quote!(String::from(#v)))
            .unwrap_or_else(|| quote!(String::from(env!("CARGO_PKG_NAME"))));

        Self::check_parameters(&generics.params)?;

        let mut fgs = vec![];

        match input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(ref fields),
                ..
            }) => {
                let mut sub_index = 0;
                let mut pos_index = 1;

                for (id, field) in fields.named.iter().enumerate() {
                    let id = id as u64;
                    let kind = Self::detect_attr_kind(field)?;
                    let fg = if kind.is_sub() {
                        FieldGenerator::Sub({
                            let sg = SubGenerator::new(field, id, sub_index)?;

                            sub_index += 1;
                            sg
                        })
                    } else {
                        FieldGenerator::Arg({
                            let mut ag = ArgGenerator::new(field, id, kind)?;

                            if ag.need_pos_index() {
                                ag.set_pos_index(pos_index);
                                pos_index += 1;
                            }
                            ag
                        })
                    };
                    fgs.push(fg);
                }
            }
            Data::Struct(DataStruct {
                fields: Fields::Unit,
                ..
            }) => todo!(),
            _ => return Err(error(input, "Cote only support struct type")),
        }

        Ok(Self {
            field_generators: fgs,
            name,
            orig_ident,
            generics,
            configs,
            help_uid: None,
            main_uid: None,
        })
    }

    pub fn check_parameters(paras: &Punctuated<GenericParam, Comma>) -> syn::Result<()> {
        for para in paras {
            match para {
                GenericParam::Lifetime(v) => {
                    return Err(error(
                        para.span(),
                        format!("Cote not support struct with lifetime `{:?}` currently", v),
                    ))
                }
                GenericParam::Const(v) => {
                    return Err(error(
                        para.span(),
                        format!(
                            "Cote not support struct with const parameters `{:?}` currently",
                            v
                        ),
                    ))
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn detect_attr_kind(field: &Field) -> syn::Result<AttrKind> {
        let attrs = &field.attrs;
        let has_sub_cfg = attrs.iter().any(|v| v.path.is_ident(CONFIG_SUB));
        let has_arg_cfg = attrs.iter().any(|v| v.path.is_ident(CONFIG_ARG));
        let has_cmd_cfg = attrs.iter().any(|v| v.path.is_ident(CONFIG_CMD));
        let has_pos_cfg = attrs.iter().any(|v| v.path.is_ident(CONFIG_POS));

        // check in attr or in path
        if has_sub_cfg && (has_arg_cfg || has_cmd_cfg || has_pos_cfg)
            || has_arg_cfg && (has_cmd_cfg || has_pos_cfg)
            || has_cmd_cfg && has_pos_cfg
        {
            Err(error(
                field,
                "Can not support more than one configuration on same field!",
            ))
        } else if has_sub_cfg {
            Ok(AttrKind::Sub)
        } else if has_cmd_cfg || Utils::check_in_ty(&field.ty, "Cmd")? {
            Ok(AttrKind::Cmd)
        } else if has_pos_cfg || Utils::check_in_ty(&field.ty, "Pos")? {
            Ok(AttrKind::Pos)
        } else if Utils::check_in_ty(&field.ty, "Main")? {
            Ok(AttrKind::Main)
        } else {
            Ok(AttrKind::Arg)
        }
    }

    pub fn has_sub_command(&self) -> bool {
        self.field_generators.iter().any(|v| v.is_sub())
    }

    pub fn main_uid(&self) -> Option<u64> {
        self.main_uid
    }

    pub fn help_uid(&self) -> Option<u64> {
        self.help_uid
    }

    pub fn gen_impl_for_struct(&mut self) -> syn::Result<TokenStream> {
        let generics = self.generics.clone();
        let orig_ident = self.orig_ident;
        let used_generics = Self::find_generics_t(&generics, &self.field_generators)?;
        let (_, type_generics, _) = generics.split_for_impl();
        let mut ipd_generics = GenericsModifier::new(generics.clone());
        let (impl_ipd, _, where_ipd) = ipd_generics.split_for_impl_ipd(&used_generics);
        let mut esd_generics = GenericsModifier::new(generics.clone());
        let (impl_esd, _, where_esd) = esd_generics.split_for_impl_esd(&used_generics);
        let mut pi_generics = GenericsModifier::new(generics.clone());
        let (impl_pi, _, where_pi) = pi_generics.split_for_impl_pi(&used_generics);
        let parser_update = self.gen_parser_update()?;
        let try_extract = self.gen_try_extract()?;
        let parser_interface = self.gen_parser_interface(&used_generics)?;

        Ok(quote! {
            #[doc=concat!("Automatic generated by cote-derive for [`", stringify!(#orig_ident), "`].")]
            impl #impl_ipd cote::IntoParserDerive<'inv, Set, Ser> for #orig_ident #type_generics #where_ipd {
                fn update(parser: &mut cote::prelude::Parser<'inv, Set, Ser>) -> cote::Result<()> {
                    #parser_update
                }
            }

            #[doc=concat!("Automatic generated by cote-derive for [`", stringify!(#orig_ident), "`].")]
            impl #impl_esd cote::ExtractFromSetDerive<'set, Set> for #orig_ident #type_generics #where_esd {
                fn try_extract(set: &'set mut Set) -> cote::Result<Self> where Self: Sized {
                    #try_extract
                }
            }

            #[doc=concat!("Automatic generated by cote-derive for [`", stringify!(#orig_ident), "`].")]
            impl #impl_pi #orig_ident #type_generics #where_pi {
                #parser_interface
            }
        })
    }

    pub fn gen_try_extract(&mut self) -> syn::Result<TokenStream> {
        let mut mut_field = vec![];
        let mut ref_field = vec![];

        for fg in self.field_generators.iter_mut() {
            let (is_refopt, extract) = fg.gen_try_extract()?;

            if is_refopt {
                ref_field.push(extract);
            } else {
                mut_field.push(extract);
            }
        }

        Ok(quote! {
            Ok(Self {
                #(#mut_field),*
                #(#ref_field),*
            })
        })
    }

    pub fn gen_parser_update(&mut self) -> syn::Result<TokenStream> {
        let mut creates = vec![];
        let mut inserts = vec![];
        let mut handlers = vec![];
        let mut append = |OptUpdate { c, i, h }| {
            c.into_iter().for_each(|v| creates.push(v));
            i.into_iter().for_each(|v| inserts.push(v));
            h.into_iter().for_each(|v| handlers.push(v));
        };

        // fill main and help uid before we start generate
        self.gen_main_and_help_uid()?;

        for fg in self.field_generators.iter_mut() {
            append(fg.gen_option(self.help_uid)?);
        }
        if let Some(up) = self.gen_main_option()? {
            append(up);
        }
        if let Some(up) = self.gen_help_option()? {
            append(up);
        }
        Ok(quote! {
            // a convenient type for option value
            type InferedOptVal<T> = <T as cote::prelude::Infer>::Val;

            let set = parser.optset_mut();
            let ctor_name = cote::prelude::ctor_default_name();

            #(#creates)*
            #(#inserts)*
            #(#handlers)*
            Ok(())
        })
    }

    pub fn gen_main_and_help_uid(&mut self) -> syn::Result<()> {
        // we need help uid in handler of sub
        if self.configs.has_cfg(CoteKind::Fallback) || self.configs.has_cfg(CoteKind::On) {
            self.main_uid
                .get_or_insert(self.field_generators.len() as u64);
        }
        self.configs.has_cfg(CoteKind::Help).then(|| {
            let total = self.field_generators.len() + if self.main_uid.is_some() { 1 } else { 0 };
            *self.help_uid.get_or_insert(total as u64)
        });
        Ok(())
    }

    pub fn gen_main_option(&self) -> syn::Result<Option<OptUpdate>> {
        let span = self.orig_ident.span();
        let on = self.configs.find_cfg(CoteKind::On);
        let fallback = self.configs.find_cfg(CoteKind::Fallback);

        Ok(if on.is_some() || fallback.is_some() {
            // if on/fallback is set, it is safe to unwrap here
            let main_uid = self.main_uid().unwrap();
            let ident = Utils::id2opt_ident(main_uid, span);
            let uid_ident = Utils::id2opt_uid_ident(main_uid, span);
            let literal = Utils::id2uid_literal(main_uid);
            let create = Utils::gen_opt_create(
                &ident,
                Some(quote! {
                    cfg.set_name("main_option");
                    <cote::prelude::Main>::infer_fill_info(&mut cfg)?;
                }),
            )?;
            let insert = Utils::gen_opt_insert(&ident, &uid_ident, &literal)?;
            let handler = Utils::gen_opt_handler(
                &uid_ident,
                on,
                fallback,
                self.configs.find_cfg(CoteKind::Then),
            )?;

            Some(
                OptUpdate::default()
                    .with_create(create)
                    .with_insert(insert)
                    .with_handler(handler.unwrap()),
            )
        } else {
            None
        })
    }

    pub fn gen_help_option(&self) -> syn::Result<Option<OptUpdate>> {
        Ok(if self.configs.find_cfg(CoteKind::Help).is_some() {
            let span = self.orig_ident.span();
            let help_opt = self
                .configs
                .find_cfg(CoteKind::HelpOpt)
                .map(|v| v.to_token_stream())
                .unwrap_or_else(|| "--help;-h=b: Display help message".to_token_stream());

            // safe here
            let help_uid = self.help_uid().unwrap();
            let ident = Utils::id2opt_ident(help_uid, span);
            let uid_ident = Utils::id2opt_uid_ident(help_uid, span);
            let literal = Utils::id2uid_literal(help_uid);
            let create = quote! {
                let #ident = {
                    let cfg = {
                        let mut cfg = cote::prelude::ConfigBuild::build(#help_opt, set)?;

                        <bool>::infer_fill_info(&mut cfg)?;
                        cfg
                    };
                    set.ctor_mut(&ctor_name)?.new_with(cfg).map_err(Into::into)?
                };
            };
            let insert = Utils::gen_opt_insert(&ident, &uid_ident, &literal)?;
            let handler = quote! {
                // we save the original option text to `Ser`, it will use in handler of `sub`
                parser.entry(#uid_ident)?.on(
                    move |_: &mut cote::prelude::Parser<'inv, Set, Ser>, ser: &mut Ser, args: cote::prelude::ctx::Args,
                                index: cote::prelude::ctx::Index| {
                        ser.sve_insert::<cote::prelude::RawVal>(args.get(*index).cloned().unwrap());
                        Ok(Some(true))
                    }
                );
            };

            Some(
                OptUpdate::default()
                    .with_create(create)
                    .with_insert(insert)
                    .with_handler(handler),
            )
        } else {
            None
        })
    }

    pub fn gen_help_context(&self) -> syn::Result<TokenStream> {
        let head = self
            .configs
            .find_value(CoteKind::Head)
            .map(|v| quote! { String::from(#v) })
            .unwrap_or_else(|| quote! { String::from(env!("CARGO_PKG_DESCRIPTION")) });
        let foot = self
            .configs
            .find_value(CoteKind::Foot)
            .map(|v| quote! { String::from(#v) })
            .unwrap_or_else(|| quote! {
                format!("Create by {} v{}", env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_VERSION"))
            });
        let width = self
            .configs
            .find_value(CoteKind::HelpWidth)
            .map(|v| quote! { #v })
            .unwrap_or(quote! { 40usize });
        let usage_width = self
            .configs
            .find_value(CoteKind::UsageWidth)
            .map(|v| quote! { #v })
            .unwrap_or(quote! { 10usize });
        let name = &self.name;

        Ok(quote! {
            cote::prelude::HelpContext::default()
                .with_name(#name)
                .with_head(#head)
                .with_foot(#foot)
                .with_width(#width)
                .with_usagew(#usage_width)
        })
    }

    pub fn gen_parser_interface(&self, used: &[&Ident]) -> syn::Result<TokenStream> {
        let sub_parsers = self.gen_sub_parsers()?;
        let help_context = self.gen_help_context()?;
        let policy_def_ty = self.gen_policy_ty(true)?;
        let policy_ret_ty = self.gen_policy_ty(false)?;
        let policy_setting_mod = self.gen_policy_setting_mod()?;
        let method_calls = self.gen_method_call()?;
        let parser_name = &self.name;
        let abort = self.configs.find_cfg(CoteKind::AbortHelp);
        let help = self.configs.find_cfg(CoteKind::Help);
        let alter = GenericsModifier::gen_alter_for_ty(used);
        let fetch_generics =
            GenericsModifier::gen_fetch_for_ty(used, quote!('set), quote!(Set), true);
        let fetch_alter = {
            let fetch = GenericsModifier::gen_fetch_for_ty(
                used,
                quote!('set),
                quote!(cote::prelude::ASet),
                true,
            );

            quote! { #alter #fetch }
        };
        let sync_rctx_from_ret =
            Utils::gen_sync_ret(abort.is_some(), help.is_some(), self.help_uid())?;
        let where_clause = quote! {
            P::Error: Into<cote::Error>,
            P::Ret: cote::prelude::Status,
            Ser: cote::prelude::ServicesValExt + Default + 'inv,
            cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default,
            <Set as cote::prelude::OptParser>::Output: cote::prelude::Information,
            Set: cote::prelude::Set + cote::prelude::OptParser + cote::prelude::OptValidator + cote::prelude::SetValueFindExt + Default + 'inv,
            P: cote::prelude::Policy<
                Set = cote::prelude::Parser<'inv, Set, Ser>,
                Ser = Ser,
                Inv<'inv> = cote::prelude::Invoker<'inv, cote::prelude::Parser<'inv, Set, Ser>, Ser>
            > + cote::prelude::APolicyExt<P> + cote::prelude::PolicySettings + Default,
            #alter
            #fetch_generics
        };

        Ok(quote! {
            #[doc(hidden)]
            pub fn new_help_context() -> cote::prelude::HelpContext {
                #help_context
            }

            #[doc(hidden)]
            pub fn sync_rctx<'a, Set, Ret>(rctx: &'a mut cote::prelude::RunningCtx, ret: &cote::Result<Ret>, set: &Set)
            -> cote::Result<&'a mut cote::prelude::RunningCtx>
                where Set: cote::prelude::SetValueFindExt, Ret: cote::prelude::Status,
                    cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default {
                #sync_rctx_from_ret
                Ok(rctx)
            }

            pub fn into_parser<'inv>() -> cote::Result<cote::prelude::Parser<'inv, cote::prelude::ASet, cote::prelude::ASer>>
            where #fetch_alter {
                Self::into_parser_with::<cote::prelude::ASet, cote::prelude::ASer>()
            }

            pub fn into_parser_with<'inv, Set, Ser>() -> cote::Result<cote::prelude::Parser<'inv, Set, Ser>>
            where
                Ser: cote::prelude::ServicesValExt + Default + 'inv,
                cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default,
                <Set as cote::prelude::OptParser>::Output: cote::prelude::Information,
                Set: cote::prelude::Set + cote::prelude::OptParser + cote::prelude::OptValidator + cote::prelude::SetValueFindExt + Default + 'inv,
                #alter
                #fetch_generics {
                let mut parser = <Self as cote::IntoParserDerive<'inv, Set, Ser>>::into_parser()?;

                #sub_parsers

                Ok(parser.with_name(#parser_name))
            }

            pub fn into_policy<'inv>() -> #policy_def_ty {
                Self::into_policy_with()
            }

            pub fn into_policy_with<'inv, Set, Ser>() -> #policy_ret_ty {
                let mut policy: #policy_ret_ty = Default::default();
                Self::apply_policy_settings(&mut policy);
                policy
            }

            pub fn apply_policy_settings(policy: &mut impl cote::prelude::PolicySettings) {
                let style_manager = policy.style_manager_mut();
                #policy_setting_mod
            }

            pub fn parse_args_with<'inv, Set, Ser, P>(args: cote::prelude::Args, policy: &mut P)
                -> cote::Result<cote::prelude::CoteRes<&mut P, P>> where #where_clause {
                let mut parser = Self::into_parser_with::<'inv, Set, Ser>()?;

                // call on parser or policy set by user
                #(#method_calls)*

                let mut rctx = cote::prelude::RunningCtx::default();

                // setup a new running ctx, add name of current parser
                rctx.add_name(#parser_name);
                parser.set_rctx(rctx);

                let args = cote::prelude::ARef::new(args);
                let ret = cote::prelude::PolicyParser::parse_policy(&mut parser, args, policy);
                let mut rctx = parser.take_rctx()?;

                // process help
                if !rctx.display_help() {
                    Self::sync_rctx::<Set, _>(&mut rctx, &ret, parser.optset())?;
                    if rctx.display_help() {
                        rctx.set_help_context(Self::new_help_context());
                    }
                }

                // insert back
                parser.set_rctx(rctx);
                let mut rctx = parser.rctx()?;

                if rctx.display_help() {
                    let names = rctx.names().iter().map(|v|v.as_str()).collect::<Vec<&str>>();
                    let help_context = rctx.help_context().unwrap();
                    let exit = rctx.exit();

                    parser.display_sub_help(names, &help_context)?;

                    // process exit, or force not exit
                    if exit {
                        std::process::exit(0);
                    }
                }

                Ok(cote::prelude::CoteRes{ ret: ret?, parser, policy })
            }

            pub fn parse_args<'inv>(args: cote::prelude::Args) -> cote::Result<cote::prelude::CoteRes<#policy_def_ty, #policy_def_ty>>
                where #fetch_alter {
                let mut policy = Self::into_policy();
                let cote::prelude::CoteRes { ret, parser, .. } = Self::parse_args_with(args, &mut policy)?;

                Ok(cote::prelude::CoteRes{ ret, parser, policy })
            }

            pub fn parse(args: cote::prelude::Args) -> cote::Result<Self>
            where #fetch_alter {
                let cote::prelude::CoteRes { mut ret, mut parser, .. } = Self::parse_args(args)?;
                let okay = ret.status();

                if okay {
                    <Self as cote::ExtractFromSetDerive::<cote::prelude::ASet>>::try_extract(parser.optset_mut())
                }
                else {
                    let mut rctx = parser.take_rctx()?;
                    let mut error = ret.take_failure();

                    if let Some(chain_error) = rctx.chain_error() {
                        error = error.cause_by(chain_error);
                    }
                    let mut failed_info = rctx.take_failed_info();
                    let (command, ret) = failed_info.last_mut()
                        .map(|v|(Some(v.name.as_str()), &mut v.retval))
                        .unwrap_or((None, &mut ret));
                    let e = {
                        let ctx = ret.take_ctx();
                        let args = ctx.orig_args()[1..]
                                    .iter()
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>()
                                    .join(", ");
                        let inner_ctx = ctx.inner_ctx().ok();
                        let failed_msg = if let Some(command) = command {
                            format!("Parsing command `{}`", command)
                        }
                        else {
                            format!("Parsing arguments `{}`", args)
                        };
                        let inner_ctx = if let Some(inner_ctx) = inner_ctx {
                            format!("{}", inner_ctx)
                        } else {
                            "None".to_owned()
                        };

                        // return failure with more detail error message
                        cote::prelude::raise_failure!("{} failed: {}", failed_msg, inner_ctx).cause_by(error)
                    };

                    Err(e)
                }
            }

            pub fn parse_env_args_with<'inv, Set, Ser, P>(policy: &mut P) -> cote::Result<cote::prelude::CoteRes<&mut P, P>>
                where #where_clause {
                Self::parse_args_with(cote::prelude::Args::from_env(), policy)
            }

            pub fn parse_env_args<'inv>() -> cote::Result<cote::prelude::CoteRes<#policy_def_ty, #policy_def_ty>>
                where #fetch_alter {
                Self::parse_args(cote::prelude::Args::from_env())
            }

            pub fn parse_env() -> cote::Result<Self>
            where #fetch_alter {
                Self::parse(cote::prelude::Args::from_env())
            }
        })
    }

    /// Insert sub parsers to main parser.
    pub fn gen_sub_parsers(&self) -> syn::Result<Option<TokenStream>> {
        let mut sub_parsers = vec![];

        for fg in self.field_generators.iter() {
            if let FieldGenerator::Sub(sg) = fg {
                let inner_ty = sg.inner_ty();
                let parser_name = sg.name();

                sub_parsers.push(quote! {
                    parser.add_parser(<#inner_ty>::into_parser_with::<Set, Ser>()?.with_name(#parser_name));
                });
            }
        }
        Ok((!sub_parsers.is_empty()).then(|| {
            quote! {
                #(#sub_parsers)*
            }
        }))
    }

    pub fn gen_policy_ty(&self, default: bool) -> syn::Result<TokenStream> {
        let policy_cfg = self.configs.find_cfg(CoteKind::Policy);
        let ty_generator: fn(&str) -> Option<TokenStream> = if default {
            Utils::gen_policy_default_ty
        } else {
            Utils::gen_policy_ty
        };

        Ok(if let Some(policy_cfg) = policy_cfg {
            let policy_name = policy_cfg.value().to_token_stream().to_string();
            let policy_ty = policy_cfg.value();

            ty_generator(&policy_name).unwrap_or_else(|| {
                if default {
                    quote! { <#policy_ty>::<'inv, cote::prelude::ASet, cote::prelude::ASer> }
                } else {
                    quote! { <#policy_ty>::<'inv, Set, Ser> }
                }
            })
        } else if self.has_sub_command() {
            ty_generator(POLICY_PRE).unwrap()
        } else {
            ty_generator(POLICY_FWD).unwrap()
        })
    }

    pub fn gen_policy_setting_mod(&self) -> syn::Result<TokenStream> {
        let enable_combine = self
            .configs
            .has_cfg(CoteKind::Combine)
            .then_some(quote! { style_manager.push(cote::prelude::UserStyle::CombinedOption);});
        let enable_embedded_plus = self
            .configs
            .has_cfg(CoteKind::EmbeddedPlus)
            .then_some(quote! { style_manager.push(cote::prelude::UserStyle::EmbeddedValuePlus);});
        let enable_flag = self
            .configs
            .has_cfg(CoteKind::Flag)
            .then_some(quote! { style_manager.push(cote::prelude::UserStyle::Flag); });
        let enable_overload = self
            .configs
            .has_cfg(CoteKind::Overload)
            .then_some(quote! { cote::prelude::PolicySettings::set_overload(policy, true); });
        let mod_strict = self.configs.find_value(CoteKind::Strict).map(|v| {
            quote! {
                cote::prelude::PolicySettings::set_strict(policy, #v);
            }
        });
        let mut nodelays = vec![];

        for fg in self.field_generators.iter().filter(|v| v.is_arg()) {
            if let Some(ret) = fg.as_arg().gen_nodelay_setting()? {
                nodelays.push(ret);
            }
        }

        Ok(quote! {
            #enable_combine
            #enable_embedded_plus
            #enable_flag
            #enable_overload
            #mod_strict
            #(#nodelays)*
        })
    }

    pub fn gen_method_call(&self) -> syn::Result<Vec<TokenStream>> {
        let span = self.orig_ident.span();
        let mut ret = vec![];

        for config in self.configs.iter() {
            if let CoteKind::MethodCall(method) = config.kind() {
                let method = Ident::new(method, span);
                let value = config.value().clone();
                let (caller, args) = value.split_call_args(span)?;
                let caller_str = caller.to_token_stream().to_string();

                match caller_str.as_str() {
                    "parser" | "policy" => {
                        ret.push(quote! {
                            #caller.#method(#args);
                        });
                    }
                    _ => {
                        let args = config.value();

                        ret.push(quote! {
                            #method(#args);
                        });
                    }
                }
            }
        }
        Ok(ret)
    }

    pub fn find_generics_t<'b>(
        _self: &'b Generics,
        fields: &[FieldGenerator],
    ) -> syn::Result<Vec<&'b Ident>> {
        let mut ret = vec![];

        for param in _self.params.iter() {
            if let syn::GenericParam::Type(ty_param) = param {
                let ident = &ty_param.ident;

                if fields.iter().any(|v| {
                    Utils::check_in_ty(v.orig_ty(), &ident.to_string()).unwrap_or_default()
                }) {
                    ret.push(ident);
                }
            }
        }

        Ok(ret)
    }
}

#[derive(Debug)]
pub enum FieldGenerator<'a> {
    Sub(SubGenerator<'a>),
    Arg(ArgGenerator<'a>),
}

impl<'a> FieldGenerator<'a> {
    pub fn is_sub(&self) -> bool {
        matches!(self, Self::Sub(_))
    }

    pub fn is_arg(&self) -> bool {
        matches!(self, Self::Arg(_))
    }

    pub fn as_arg(&self) -> &ArgGenerator<'a> {
        match self {
            FieldGenerator::Sub(_) => panic!("Not a ArgGenerator"),
            FieldGenerator::Arg(ag) => ag,
        }
    }

    // pub fn as_sub(&self) -> &SubGenerator<'a> {
    //     match self {
    //         FieldGenerator::Sub(sg) => sg,
    //         FieldGenerator::Arg(_) => panic!("Not a SubGenerator"),
    //     }
    // }

    pub fn orig_ty(&self) -> &Type {
        match self {
            FieldGenerator::Sub(sg) => sg.ty(),
            FieldGenerator::Arg(ag) => ag.ty(),
        }
    }

    pub fn gen_option(&mut self, help_uid: Option<u64>) -> syn::Result<OptUpdate> {
        match self {
            FieldGenerator::Sub(sg) => sg.gen_opt_update(help_uid),
            FieldGenerator::Arg(ag) => ag.gen_opt_update(),
        }
    }

    pub fn gen_try_extract(&mut self) -> syn::Result<(bool, TokenStream)> {
        match self {
            FieldGenerator::Sub(sg) => sg.gen_try_extract(),
            FieldGenerator::Arg(ag) => ag.gen_try_extract(),
        }
    }
}
