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
use crate::r#gen::GenericsModifier;

use super::arg::ArgGenerator;
use super::sub::SubGenerator;
use super::AttrKind;
use super::OptUpdate;
use super::Utils;
use super::CONFIG_ARG;
use super::CONFIG_CMD;
use super::CONFIG_POS;
use super::CONFIG_SUB;
use super::HELP_OPTION;
use super::POLICY_FWD;

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
            }) => {}
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
        let has_sub_cfg = attrs.iter().any(|v| v.path().is_ident(CONFIG_SUB));
        let has_arg_cfg = attrs.iter().any(|v| v.path().is_ident(CONFIG_ARG));
        let has_cmd_cfg = attrs.iter().any(|v| v.path().is_ident(CONFIG_CMD));
        let has_pos_cfg = attrs.iter().any(|v| v.path().is_ident(CONFIG_POS));

        // check in attr or in path
        if has_sub_cfg && (has_arg_cfg || has_cmd_cfg || has_pos_cfg)
            || has_arg_cfg && (has_cmd_cfg || has_pos_cfg)
            || has_cmd_cfg && has_pos_cfg
        {
            Err(error(
                field,
                "Can not support more than one configuration on same field!",
            ))
        } else if has_arg_cfg {
            Ok(AttrKind::Arg)
        } else if has_sub_cfg {
            Ok(AttrKind::Sub)
        } else if has_cmd_cfg {
            Ok(AttrKind::Cmd)
        } else if has_pos_cfg || Utils::check_in_ty(&field.ty, "Pos")? {
            Ok(AttrKind::Pos)
        } else if Utils::check_in_ty(&field.ty, "Main")? {
            Ok(AttrKind::Main)
        } else if Utils::check_in_ty(&field.ty, "Cmd")? {
            Ok(AttrKind::Cmd)
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
            impl #impl_ipd cote::IntoParserDerive<'inv, Set> for #orig_ident #type_generics #where_ipd {
                fn update(parser: &mut cote::prelude::Parser<'inv, Set>) -> cote::Result<()> {
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
                    cote::prelude::ConfigValue::set_name(&mut cfg, "main_option");
                    <cote::prelude::Main as cote::prelude::Infer>::infer_fill_info(&mut cfg)?;
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
                .unwrap_or_else(|| HELP_OPTION.to_token_stream());

            // safe here
            let help_uid = self.help_uid().unwrap();
            let ident = Utils::id2opt_ident(help_uid, span);
            let uid_ident = Utils::id2opt_uid_ident(help_uid, span);
            let literal = Utils::id2uid_literal(help_uid);
            let create = quote! {
                let #ident = {
                    let cfg = {
                        let mut cfg = cote::prelude::ConfigBuild::build(#help_opt, set)?;

                        <bool as cote::prelude::Infer>::infer_fill_info(&mut cfg)?;
                        cfg
                    };
                    cote::prelude::Ctor::new_with(cote::prelude::SetExt::ctor_mut(set, &ctor_name)?, cfg).map_err(Into::into)?
                };
            };
            let insert = Utils::gen_opt_insert(&ident, &uid_ident, &literal)?;
            let handler = quote! {
                // we save the original option text to `Set`, it will use in handler of `sub`
                parser.entry(#uid_ident)?.on(
                    move |set: &mut cote::prelude::Parser<'inv, Set>, ctx: &mut cote::prelude::Ctx| {
                        let args = ctx.args();
                        let index = ctx.idx()?;

                        cote::prelude::AppStorage::set_app_data( set,
                            cote::prelude::HideValue(args[index].to_os_string())
                        );
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
        let shell_where_clause = quote! {
            cote::prelude::SetOpt<S>: cote::prelude::Opt + 'static,
            cote::prelude::SetCfg<S>: cote::prelude::ConfigValue + Default,
            S: cote::prelude::Set + cote::prelude::OptValidator + cote::prelude::SetValueFindExt,
        };
        let try_complete_impl = self.gen_try_complete_with()?;
        let inject_values_func = self.gen_inject_completion_values(&shell_where_clause);
        let parser_name = &self.name;
        let abort = self.configs.find_cfg(CoteKind::AbortHelp);
        let help = self.configs.find_cfg(CoteKind::Help);
        let not_exit = self.configs.find_cfg(CoteKind::NotExit);
        let exit_after_display_help = not_exit.is_none();
        let infer_override = GenericsModifier::gen_inferoverride_for_ty(used);
        let fetch_generics = GenericsModifier::gen_fetch_for_ty(used, quote!(Set));
        let fetch_code = {
            let fetch = GenericsModifier::gen_fetch_for_ty(used, quote!(cote::prelude::CoteSet));

            quote! { #infer_override  #fetch }
        };
        let sync_rctx_from_ret = Utils::gen_sync_ret(
            self.has_sub_command(),
            abort.is_some(),
            help.is_some(),
            self.help_uid(),
        )?;
        let where_clause = quote! {
            P::Error: Into<cote::Error>,
            P::Ret: cote::prelude::Status,
            cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default,
            Set: cote::prelude::Set +
            cote::prelude::OptParser<Output: cote::prelude::Information> + cote::prelude::OptValidator
            + cote::prelude::PrefixedValidator + cote::prelude::SetValueFindExt + Default + 'inv,
            P: cote::prelude::Policy<
                Set = cote::prelude::Parser<'inv, Set>,
                Inv<'inv> = cote::prelude::Invoker<'inv, cote::prelude::Parser<'inv, Set>>
            > + cote::prelude::PolicySettings + Default,
            #fetch_generics
        };

        Ok(quote! {
            /// Return a new help context using for display help message.
            pub fn new_help_context() -> cote::prelude::HelpContext {
                #help_context
            }

            // inject the values into shell completion manager
            #inject_values_func

            #[doc(hidden)]
            pub fn sync_rctx<'a, Set, Ret>(rctx: &'a mut cote::prelude::RunningCtx, ret: &cote::Result<Ret>, set: &Set, sub_parser: bool)
            -> cote::Result<&'a mut cote::prelude::RunningCtx>
                where Set: cote::prelude::SetValueFindExt, Ret: cote::prelude::Status,
                    cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default {
                #sync_rctx_from_ret
                Ok(rctx)
            }

            pub fn into_parser<'inv>() -> cote::Result<cote::prelude::Parser<'inv, cote::prelude::CoteSet>>
            where #fetch_code {
                Self::into_parser_with::<cote::prelude::CoteSet>()
            }

            pub fn into_parser_with<'inv, Set>() -> cote::Result<cote::prelude::Parser<'inv, Set>>
            where
                cote::prelude::SetCfg<Set>: cote::prelude::ConfigValue + Default,
                Set: cote::prelude::Set +
                cote::prelude::OptParser<Output: cote::prelude::Information> + cote::prelude::OptValidator
                + cote::prelude::PrefixedValidator + cote::prelude::SetValueFindExt + Default + 'inv,
                #fetch_generics {
                let mut parser = <Self as cote::IntoParserDerive<'inv, Set>>::into_parser()?;

                #sub_parsers

                Ok(parser.with_name(#parser_name))
            }

            pub fn into_policy<'inv>() -> #policy_def_ty {
                Self::into_policy_with()
            }

            pub fn into_policy_with<'inv, Set>() -> #policy_ret_ty {
                let mut policy: #policy_ret_ty = Default::default();
                Self::apply_policy_settings(&mut policy);
                policy
            }

            pub fn apply_policy_settings(policy: &mut impl cote::prelude::PolicySettings) {
                let style_manager = policy.style_manager_mut();
                #policy_setting_mod
            }

            pub fn parse_args_with<'inv, Set, P>(args: cote::prelude::Args, policy: &mut P)
                -> cote::Result<cote::prelude::CoteRes<&mut P, P>> where #where_clause {
                let mut parser = Self::into_parser_with::<'inv, Set>()?;

                // call on parser or policy set by user
                #(#method_calls)* // todo! do we need apply this in sub handler ?

                // setup a new running ctx, set name of parser
                parser.running_ctx().set_name(#parser_name);

                let ret = cote::prelude::PolicyParser::parse_policy(&mut parser, args, policy);
                let mut rctx = std::mem::take(parser.running_ctx());

                // process help
                if !rctx.display_help() {
                    Self::sync_rctx::<Set, _>(&mut rctx, &ret, parser.optset(), false)?;
                    if rctx.display_help() {
                        rctx.set_help_context(Self::new_help_context());
                    }
                }

                // display help
                if rctx.display_help() {
                    let names: Vec<_> = std::iter::once(rctx.name())
                           .chain(rctx.frames().iter().map(|v|v.name.as_str())).collect();
                    // we set the help context if we need display help, so just unwrap it
                    let help_context = rctx.help_context().unwrap();
                    let exit = rctx.exit();

                    cote::prelude::HelpDisplay::display_sub(&parser, names, &help_context)?;

                    // process exit, or force not exit
                    if exit && #exit_after_display_help {
                        std::process::exit(0);
                    }
                }

                // insert back running ctx
                parser.set_running_ctx(rctx);

                Ok(cote::prelude::CoteRes{ ret: ret?, parser, policy })
            }

            pub fn parse_args<'inv>(args: cote::prelude::Args) -> cote::Result<cote::prelude::CoteRes<#policy_def_ty, #policy_def_ty>>
                where #fetch_code {
                let mut policy = Self::into_policy();
                let cote::prelude::CoteRes { ret, parser, .. } = Self::parse_args_with(args, &mut policy)?;

                Ok(cote::prelude::CoteRes{ ret, parser, policy })
            }

            pub fn parse(args: cote::prelude::Args) -> cote::Result<Self>
            where #fetch_code {
                let cote::prelude::CoteRes { mut ret, mut parser, .. } = Self::parse_args(args)?;

                Self::from(ret, parser)
            }

            pub fn parse_env_args_with<'inv, Set, P>(policy: &mut P) -> cote::Result<cote::prelude::CoteRes<&mut P, P>>
                where #where_clause {
                Self::parse_args_with(cote::prelude::Args::from_env(), policy)
            }

            pub fn parse_env_args<'inv>() -> cote::Result<cote::prelude::CoteRes<#policy_def_ty, #policy_def_ty>>
                where #fetch_code {
                Self::parse_args(cote::prelude::Args::from_env())
            }

            pub fn parse_env() -> cote::Result<Self>
            where #fetch_code {
                Self::parse(cote::prelude::Args::from_env())
            }

            // add shell completion if shellcomp set
            pub fn try_complete() -> cote::Result<()> {
                Self::try_complete_with(Self::into_parser()?)
            }

            pub fn try_complete_with<'inv, S>(parser: cote::prelude::Parser<'inv, S>)
                -> cote::Result<()> where #shell_where_clause {
                #try_complete_impl
            }

            pub fn from<'inv, S>(mut ret: cote::prelude::Return, mut parser: cote::prelude::Parser<'inv, S>) -> cote::Result<Self> where S: cote::prelude::SetValueFindExt,
            cote::prelude::SetCfg<S>: cote::prelude::ConfigValue + Default {
                if let Some(mut error) = ret.take_failure() {
                    let mut rctx = std::mem::take(parser.running_ctx());
                    let mut failures = rctx.frames_mut().iter_mut().map(|v|v.failure.as_mut().unwrap());
                    let ctx = ret.take_ctx();
                    let mut cmd = None;
                    let mut ret = Some(&mut ret);

                    // chain the error in the frames
                    for failure in failures {
                        if let Some(sub_error) = failure.retval.take_failure() {
                            error = error.cause_by(sub_error);
                        }
                        cmd = Some(failure.cmd.as_str());
                        ret = Some(&mut failure.retval);
                    }

                    // construct error message
                    let e = {
                        let args = ctx.orig[1..].iter()
                                    .map(|v|std::path::Path::new(v).display())
                                    .map(|v|v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                        let guess = ctx.guess;
                        let failed_msg = if let Some(cmd) = cmd {
                            format!("Parsing command `{}`", cmd)
                        }
                        else {
                            format!("Parsing arguments `{}`", args)
                        };
                        let guess = if let Some(guess) = guess {
                            format!("{:?}", guess)
                        } else {
                            "None".to_owned()
                        };

                        // return failure with more detail error message
                        cote::prelude::failure!("{} failed: {}", failed_msg, guess).cause_by(error)
                    };

                    Err(e)
                }
                else {
                    <Self as cote::ExtractFromSetDerive::<S>>::try_extract(parser.optset_mut())
                }
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
                    parser.add_parser(<#inner_ty>::into_parser_with::<Set>()?.with_name(#parser_name));
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
                    quote! { #policy_ty<'inv, cote::prelude::CoteSet> }
                } else {
                    quote! { #policy_ty<'inv, Set> }
                }
            })
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
        let enable_overload = self.configs.find_value(CoteKind::Overload).map(|v| {
            quote! { cote::prelude::PolicySettings::set_overload(policy, #v); }
        });
        // if we have sub command, enable the prepolicy setting
        let enable_prepolicy = self
            .configs
            .find_value(CoteKind::PrePolicy)
            .map(|v| {
                quote! { cote::prelude::PolicySettings::set_prepolicy(policy, #v); }
            })
            .or(self
                .has_sub_command()
                .then_some(quote! { cote::prelude::PolicySettings::set_prepolicy(policy, true); }));
        let enable_strict = self.configs.find_value(CoteKind::Strict).map(|v| {
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
            #enable_prepolicy
            #enable_strict
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
                    "policy" => {
                        ret.push(quote! {
                            #method::<P>(policy, #args)?;
                        });
                    }
                    "parser" => {
                        ret.push(quote! {
                            #method::<Set>(&mut parser, #args)?;
                        });
                    }
                    _ => {
                        let args = config.value();

                        ret.push(quote! {
                            #method(#args)?;
                        });
                    }
                }
            }
        }
        Ok(ret)
    }

    pub fn gen_try_complete_with(&self) -> syn::Result<TokenStream> {
        if self.configs.has_cfg(CoteKind::ShellCompletion) {
            let binname = &self.name;

            Ok(quote! {
                let ccli = cote::shell::get_complete_cli()?;
                let binary_name = #binname;

                if ccli.write_stdout(&binary_name, &binary_name).is_ok() {
                    return Ok(())
                }
                ccli.complete(|shell| {
                    let mut ctx = ccli.get_context()?;
                    let mut manager = cote::shell::CompletionManager::new(parser);

                    shell.set_buff(std::io::stdout());
                    Self::inject_completion_values(&mut manager)?;
                    cote::shell::shell::Complete::complete(&manager, shell, &mut ctx)?;
                    Ok(())
                })?;
                Ok(())
            })
        } else {
            Ok(quote! { Err(cote::prelude::error!("not support shell completion")) })
        }
    }

    pub fn gen_inject_completion_values(&self, where_clause: &TokenStream) -> Option<TokenStream> {
        let force_inject = self.configs.has_cfg(CoteKind::ShellCompletion);
        let mut arg_injects = vec![];
        let mut sub_injects = vec![];

        for fg in self.field_generators.iter() {
            match fg {
                FieldGenerator::Sub(sg) => {
                    if let Some(sub_inject) = sg.gen_inject_completion_values() {
                        sub_injects.push(sub_inject);
                    }
                }
                FieldGenerator::Arg(ag) => {
                    if let Some(arg_inject) = ag.gen_inject_completion_values() {
                        arg_injects.push(arg_inject);
                    }
                }
            }
        }

        if force_inject || !arg_injects.is_empty() || !sub_injects.is_empty() {
            Some(quote! {
                #[doc(hidden)]
                pub fn inject_completion_values<'inv, S>(manager: &mut cote::shell::CompletionManager<'inv, S>)
                    -> cote::Result<()> where #where_clause {
                    #(#arg_injects)*
                    #(#sub_injects)*
                    Ok(())
                }
            })
        } else {
            None
        }
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

#[allow(clippy::large_enum_variant)]
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
