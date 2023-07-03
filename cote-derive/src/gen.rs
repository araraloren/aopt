mod arg;
mod cote;
mod sub;

use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Attribute;
use syn::DataStruct;
use syn::DeriveInput;
use syn::Field;
use syn::Fields;
use syn::GenericArgument;
use syn::GenericParam;
use syn::Index;
use syn::Lit;
use syn::PathArguments;
use syn::Token;
use syn::Type;
use syn::WherePredicate;

const HELP_OPTION_SHORT: &str = "-h";
const HELP_OPTION_NAME: &str = "--help";
const HELP_OPTION_HELP: &str = "Display help message";
const POLICY_PRE: &str = "pre";
const POLICY_FWD: &str = "fwd";
const POLICY_DELAY: &str = "delay";
const CONFIG_ARG: &str = "arg";
const CONFIG_POS: &str = "pos";
const CONFIG_CMD: &str = "cmd";
const APP_POSTFIX: &str = "InternalApp";

pub use self::arg::ArgGenerator;
pub use self::cote::CoteGenerator;
pub use self::sub::SubGenerator;

pub type OptUpdate = (
    Option<TokenStream>,
    Option<TokenStream>,
    Option<TokenStream>,
);

#[derive(Debug, Default)]
pub struct Update {
    pub create: Option<TokenStream>,

    pub insert: Option<TokenStream>,

    pub handler: Option<TokenStream>,
}

#[derive(Debug)]
pub struct Analyzer<'a> {
    cote_generator: CoteGenerator<'a>,

    arg_generator: Vec<ArgGenerator<'a>>,

    sub_generator: Vec<SubGenerator<'a>>,
}

impl<'a> Analyzer<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        match input.data {
            syn::Data::Struct(DataStruct {
                fields: Fields::Named(ref fields),
                ..
            }) => {
                let mut cote_generator = CoteGenerator::new(input)?;
                let mut arg_generator = vec![];
                let mut sub_generator = vec![];
                let mut sub_app_idx = 0;
                let mut pos_arg_idx = 1;

                for field in fields.named.iter() {
                    if check_if_has_sub_cfg(field)? {
                        sub_generator.push(SubGenerator::new(field, sub_app_idx)?);
                        cote_generator.set_has_sub_command(true);
                        sub_app_idx += 1;
                    } else {
                        let arg = ArgGenerator::new(field, pos_arg_idx)?;

                        if arg.has_pos_id() {
                            pos_arg_idx += 1;
                        }
                        arg_generator.push(arg);
                    }
                }
                Ok(Self {
                    arg_generator,
                    cote_generator,
                    sub_generator,
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

    pub fn gen_all(&self) -> syn::Result<TokenStream> {
        let ident = self.cote_generator.get_ident();
        let (params, where_predicate) = self.cote_generator.split_for_impl();
        let (impl_parser, type_parser, where_parser) =
            self.gen_impl_for_parser(params, where_predicate);
        let (impl_ip, type_ip, where_ip) = self.gen_impl_for_ip(params, where_predicate);
        let (impl_sd, type_sd, where_sd) = self.gen_impl_for_sd(params, where_predicate);
        let parser_update = self.gen_parser_update()?;
        let try_extract = self.gen_try_extract()?;
        let parser_interface = self.gen_parser_interface()?;
        let internal_helper = self.gen_internal_helper_struct()?;

        Ok(quote! {
            #[doc=concat!("Automatic generated by cote-derive for [`", stringify!(#ident), "`].")]
            impl #impl_ip cote::IntoParserDerive<'inv, Set, Ser> for #ident #type_ip #where_ip {
                fn update(parser: &mut cote::Parser<'inv, Set, Ser>) -> Result<(), cote::CoteError> {
                    #parser_update
                }
            }

            #[doc=concat!("Automatic generated by cote-derive for [`", stringify!(#ident), "`].")]
            impl #impl_sd cote::ExtractFromSetDerive<'set, Set> for #ident #type_sd #where_sd {
                fn try_extract(set: &'set mut Set) -> Result<Self, cote::CoteError> where Self: Sized {
                    #try_extract
                }
            }

            #[doc=concat!("Automatic generated by cote-derive for [`", stringify!(#ident), "`].")]
            impl #impl_parser #ident #type_parser #where_parser {
                #parser_interface
            }

            #internal_helper
        })
    }

    pub fn gen_impl_for_sd(
        &self,
        params: &Punctuated<GenericParam, Token![,]>,
        where_predicate: Option<&Punctuated<WherePredicate, Token![,]>>,
    ) -> (TokenStream, TokenStream, TokenStream) {
        (
            if params.is_empty() {
                quote! {
                    <'set, Set>
                }
            } else {
                quote! {
                    <'set, #params, Set>
                }
            },
            if params.is_empty() {
                quote! {}
            } else {
                quote! {
                    <#params>
                }
            },
            self.gen_where_for_set_derive(where_predicate),
        )
    }

    pub fn gen_impl_for_ip(
        &self,
        params: &Punctuated<GenericParam, Token![,]>,
        where_predicate: Option<&Punctuated<WherePredicate, Token![,]>>,
    ) -> (TokenStream, TokenStream, TokenStream) {
        (
            if params.is_empty() {
                quote! {
                    <'inv, Set, Ser>
                }
            } else {
                quote! {
                    <'inv, #params, Set, Ser>
                }
            },
            if params.is_empty() {
                quote! {}
            } else {
                quote! {
                    <#params>
                }
            },
            self.gen_where_for_into_parser(where_predicate),
        )
    }

    pub fn gen_impl_for_parser(
        &self,
        params: &Punctuated<GenericParam, Token![,]>,
        where_predicate: Option<&Punctuated<WherePredicate, Token![,]>>,
    ) -> (TokenStream, TokenStream, TokenStream) {
        (
            if params.is_empty() {
                quote! {}
            } else {
                quote! {
                    <#params>
                }
            },
            if params.is_empty() {
                quote! {}
            } else {
                quote! {
                    <#params>
                }
            },
            if let Some(where_predicate) = where_predicate {
                quote! { where #where_predicate }
            } else {
                quote! {}
            },
        )
    }

    pub fn gen_where_for_set_derive(
        &self,
        where_predicate: Option<&Punctuated<WherePredicate, Token![,]>>,
    ) -> TokenStream {
        let default_where = quote! {
            where Set: cote::SetValueFindExt,
        };
        if let Some(where_predicate) = where_predicate {
            quote! {
                #default_where
                #where_predicate
            }
        } else {
            default_where
        }
    }

    pub fn gen_where_for_into_parser(
        &self,
        where_predicate: Option<&Punctuated<WherePredicate, Token![,]>>,
    ) -> TokenStream {
        let default_where = quote! {
            where
            Ser: cote::ServicesValExt + Default + 'inv,
            cote::SetCfg<Set>: cote::Config + cote::ConfigValue + Default,
            Set: cote::Set + cote::OptParser + cote::OptValidator + cote::SetValueFindExt + Default + 'inv,
        };
        if let Some(where_predicate) = where_predicate {
            quote! {
                #default_where
                #where_predicate
            }
        } else {
            default_where
        }
    }

    pub fn gen_try_extract(&self) -> syn::Result<TokenStream> {
        let mut mut_field = vec![];
        let mut ref_field = vec![];

        for field in self.arg_generator.iter() {
            let (is_refopt, ts) = field.gen_value_extract()?;

            if is_refopt {
                ref_field.push(ts);
            } else {
                mut_field.push(ts);
            }
        }
        for field in self.sub_generator.iter() {
            let (is_refopt, ts) = field.gen_field_extract()?;

            if is_refopt {
                ref_field.push(ts);
            } else {
                mut_field.push(ts);
            }
        }
        let mut ret = quote! {};

        ret.extend(mut_field.into_iter());
        ret.extend(ref_field.into_iter());
        Ok(quote! {
            Ok(Self {
                #ret
            })
        })
    }

    pub fn gen_parser_update(&self) -> syn::Result<TokenStream> {
        let mut ret = quote! {
            let set = parser.optset_mut();
            let ctor_name = cote::ctor_default_name();
            let ctor = set.ctor_mut(&ctor_name)?;
        };
        let mut create = vec![];
        let mut insert = vec![];
        let mut handler = vec![];
        let mut option_id = 0;
        let is_process_help = self.cote_generator.is_process_help();
        let mut help_uid = None;

        let mut append = |(c, i, h): OptUpdate| {
            c.into_iter().for_each(|v| create.push(v));
            i.into_iter().for_each(|v| insert.push(v));
            h.into_iter().for_each(|v| handler.push(v));
        };

        if let Some(update) = self.cote_generator.gen_main_option_update(option_id) {
            append(update);
            option_id += 1;
        }
        if let Some((uid, update)) = self.cote_generator.gen_help_option_update(option_id) {
            help_uid = Some(uid);
            append(update);
            option_id += 1;
        }
        for field in self.arg_generator.iter() {
            append(field.gen_option_update(option_id)?);
            option_id += 1;
        }
        for field in self.sub_generator.iter() {
            append(field.gen_option_update(option_id, is_process_help, help_uid.as_ref())?);
            option_id += 1;
        }
        ret.extend(create.into_iter());
        ret.extend(insert.into_iter());
        ret.extend(handler.into_iter());
        ret.extend(quote! { Ok(()) });
        Ok(ret)
    }

    pub fn gen_display_call_for_sub_help(&self) -> syn::Result<TokenStream> {
        let mut display_sub = quote! {};
        let mut display_call = quote! {};

        for sub_generator in self.sub_generator.iter() {
            let help_context_gen = sub_generator.gen_update_help_context()?;
            let internal_ty = sub_generator.gen_internal_ty()?;
            let idx = sub_generator.get_sub_index();
            let idx = Index::from(idx);

            display_sub.extend(quote! {
                if &sub_name == sub_parsers[#idx].name() {
                    let mut context = #internal_ty::<'_, (), ()>::display_ctx();
                    let context = { #help_context_gen };
                    let name_of_help = names.join(" ");
                    let optset = sub_parsers[#idx].optset();

                    return cote::display_help!(
                        optset,
                        &name_of_help, context.head(), context.foot(), context.width(), context.usagew()
                    );
                }
            });
            display_call.extend(quote! {
                if &sub_name == sub_parsers[#idx].name() {
                    let mut sub_internal_helper: #internal_ty<'_, cote::Parser<'inv, Set, Ser>, ()> = Default::default();
                    let mut policy: () = Default::default();

                    sub_internal_helper.set_inner_parser(&mut sub_parsers[#idx]);
                    sub_internal_helper.set_inner_policy(&mut policy);

                    return sub_internal_helper.display_sub_help_idx(names, idx + 1);
                }
            });
        }

        if self.sub_generator.is_empty() {
            Ok(quote! {})
        } else {
            Ok(quote! {
                let sub_name = &names[idx + 1];
                let sub_name = inner_parser.find_opt(sub_name.as_str())?.name().clone();
                let sub_parsers = inner_parser.parsers_mut();

                if idx == len - 2 {
                    #display_sub
                }
                else {
                    #display_call
                }
            })
        }
    }

    /// Insert sub parsers to main parser.
    pub fn insert_sub_parsers(&self) -> syn::Result<TokenStream> {
        let mut insert_sub_parsers = quote! {};

        for sub_generator in self.sub_generator.iter() {
            let without_option_ty = sub_generator.get_without_option_type();
            let parser_name = sub_generator.name();

            insert_sub_parsers.extend(quote! {
                parser.add_parser(<#without_option_ty>::into_parser_with::<Set, Ser>()?.with_name(#parser_name));
            });
        }

        if self.sub_generator.is_empty() {
            Ok(quote! {})
        } else {
            Ok(quote! {
                #insert_sub_parsers
            })
        }
    }

    pub fn gen_policy_setting_modifier(&self) -> TokenStream {
        let mut ret = quote! {};

        if let Some(policy_settings) = self.cote_generator.policy_settings_modifier() {
            ret.extend(policy_settings);
        }
        for arg in self.arg_generator.iter() {
            ret.extend(arg.gen_nodelay_for_delay_parser().into_iter());
        }
        ret
    }

    pub fn parser_where_clause() -> TokenStream {
        quote! {
            P::Error: Into<cote::CoteError>,
            P::Ret: cote::ReturnValStatus,
            Ser: cote::ServicesValExt + Default + 'inv,
            cote::SetCfg<Set>: cote::Config + cote::ConfigValue + Default,
            Set: cote::Set + cote::OptParser + cote::OptValidator + cote::SetValueFindExt + Default + 'inv,
            P: cote::Policy<
                Set = cote::Parser<'inv, Set, Ser>,
                Ser = Ser,
                Inv<'inv> = cote::Invoker<'inv, cote::Parser<'inv, Set, Ser>, Ser>
            > + cote::APolicyExt<P> + cote::PolicySettings + Default
        }
    }

    pub fn gen_parser_interface(&self) -> syn::Result<TokenStream> {
        let major_internal_ty = self.cote_generator.gen_internal_ty();
        let major_default_ty = self.cote_generator.gen_ret_default_policy_ty()?;
        let major_policy_ty = self.cote_generator.gen_ret_policy_ty_generics()?;
        let insert_sub_parsers = self.insert_sub_parsers()?;
        let policy_settings = self.gen_policy_setting_modifier();
        let method_call = self.cote_generator.gen_method_call()?;
        let major_parser_name = self.cote_generator.get_name();
        let where_clause = Self::parser_where_clause();

        Ok(quote! {
            pub fn into_parser<'inv>() -> Result<cote::Parser<'inv, cote::ASet, cote::ASer>, cote::CoteError> {
                Self::into_parser_with::<cote::ASet, cote::ASer>()
            }

            pub fn into_parser_with<'inv, Set, Ser>() -> Result<cote::Parser<'inv, Set, Ser>, cote::CoteError>
            where
                Ser: cote::ServicesValExt + Default + 'inv,
                cote::SetCfg<Set>: cote::Config + cote::ConfigValue + Default,
                Set: cote::Set + cote::OptParser + cote::OptValidator + cote::SetValueFindExt + Default + 'inv {
                let mut parser = <Self as cote::IntoParserDerive<'inv, Set, Ser>>::into_parser()?;

                #insert_sub_parsers

                Ok(parser.with_name(#major_parser_name))
            }

            pub fn into_policy<'inv>() -> #major_default_ty {
                Self::into_policy_with()
            }

            pub fn into_policy_with<'inv, Set, Ser>() -> #major_policy_ty {
                let mut policy: #major_policy_ty = Default::default();
                let style_manager = policy.style_manager_mut();

                #policy_settings

                policy
            }

            pub fn into_internal<'a, 'inv, Set, Ser, P>() -> #major_internal_ty<'a, cote::Parser<'inv, Set, Ser>, P>
            where P: Default, Set: Default, Ser: Default {
                Default::default()
            }

            pub fn parse_args_with<'inv, Set, Ser, P>(args: cote::Args, policy: &mut P)
            -> Result<cote::CoteRes<&mut P, P>, cote::CoteError> where #where_clause {
                let mut parser = Self::into_parser_with::<'inv, Set, Ser>()?;
                let mut helper = Self::into_internal::<'_, 'inv, Set, Ser, P>();

                #method_call
                helper.set_inner_parser(&mut parser);
                helper.set_inner_policy(policy);
                helper.set_default_rctx()?;
                helper.rctx_mut()?.add_name(#major_parser_name);

                let ret = helper.parse(cote::ARef::new(args), false);
                let rctx = helper.rctx()?;
                
                if rctx.display_sub_help() {
                    let names = rctx.names().to_vec();
                    let sub_exit = rctx.exit_sub();

                    helper.display_sub_help(&names)?;
                    if sub_exit {
                        std::process::exit(0)
                    }
                }
                else if rctx.display_help() {
                    let major_exit = rctx.exit();

                    helper.display_help()?;
                    if major_exit {
                        std::process::exit(0)
                    }
                }
                Ok(cote::CoteRes{ ret: ret?, parser: parser, policy: policy })
            }

            pub fn parse_args<'inv>(args: cote::Args)
                -> Result<cote::CoteRes<#major_default_ty, #major_default_ty>, cote::CoteError > {
                let mut policy = Self::into_policy();
                let cote::CoteRes { ret, parser, .. } = Self::parse_args_with(args, &mut policy)?;

                Ok(cote::CoteRes{ ret: ret, parser: parser, policy: policy })
            }

            pub fn parse(args: cote::Args) -> Result<Self, cote::CoteError> {
                let cote::CoteRes { mut ret, mut parser, .. } = Self::parse_args(args)?;

                if ret.status() {
                    Self::try_extract(parser.optset_mut())
                }
                else {
                    let mut rctx = parser.take_rctx()?;
                    let mut error = ret.take_failure();
                    
                    if let Some(rerror) = rctx.chain_error() {
                        error = error.cause_by(rerror);
                    }
                    let mut finfo = rctx.take_failed_info();
                    let (command, ret) = finfo.last_mut().map(|v|(Some(v.name.as_str()), &mut v.retval)).unwrap_or((None, &mut ret));
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
                        cote::raise_failure!("{} failed: {}", failed_msg, inner_ctx).cause_by(error)
                    };

                    Err(e)
                }
            }

            pub fn parse_env_args_with<'inv, Set, Ser, P>(policy: &mut P)
                -> Result<cote::CoteRes<&mut P, P>, cote::CoteError> where #where_clause {
                Self::parse_args_with(cote::Args::from_env(), policy)
            }

            pub fn parse_env_args<'inv>()
                -> Result<cote::CoteRes<#major_default_ty, #major_default_ty>, cote::CoteError> {
                Self::parse_args(cote::Args::from_env())
            }

            pub fn parse_env() -> Result<Self, cote::CoteError> {
                Self::parse(cote::Args::from_env())
            }
        })
    }

    pub fn gen_internal_helper_struct(&self) -> syn::Result<TokenStream> {
        let major_helper_ty = self.cote_generator.gen_internal_ty();
        let major_helper_define = self.cote_generator.define_helper_ty(&major_helper_ty);
        let help_display_ctx = self.cote_generator.gen_help_display_ctx();
        let call_for_sub_help = self.gen_display_call_for_sub_help()?;
        let sync_running_ctx = self.cote_generator.gen_sync_ret_value();

        Ok(quote! {
            #major_helper_define

            impl<'a, Parser, Policy> #major_helper_ty<'a, Parser, Policy> {
                pub fn display_ctx() -> cote::HelpDisplayCtx {
                    #help_display_ctx
                }
            }

            impl<'a, 'inv, Set, Ser, Policy> #major_helper_ty<'a, cote::Parser<'inv, Set, Ser>, Policy>
            where Ser: cote::ServicesValExt, Set: cote::SetValueFindExt, Policy: cote::Policy, Policy::Ret: cote::ReturnValStatus {
                pub fn sync_rctx(&mut self, ret: &Result<Policy::Ret, cote::CoteError>, sub_parser: bool)
                    -> Result<&mut Self, cote::CoteError> {
                    #sync_running_ctx
                    Ok(self)
                }
            }

            impl<'a, 'inv, Set, Ser, Policy> #major_helper_ty<'a, cote::Parser<'inv, Set, Ser>, Policy>
            where
                Policy::Error: Into<cote::CoteError>,
                Policy::Ret: cote::ReturnValStatus,
                Ser: cote::ServicesValExt,
                Set: cote::Set + cote::OptParser + cote::OptValidator + cote::SetValueFindExt,
                Policy: cote::Policy<
                        Set = cote::Parser<'inv, Set, Ser>,
                        Ser = Ser,
                        Inv<'inv> = cote::Invoker<'inv, cote::Parser<'inv, Set, Ser>, Ser>
                    > {
                pub fn parse(&mut self, args: cote::ARef<cote::Args>, sub_parser: bool) -> Result<Policy::Ret, cote::CoteError> {
                    let policy = self.policy.take().unwrap();
                    let ret = cote::PolicyParser::parse_policy(self.inner_parser_mut(), args, policy);

                    self.policy = Some(policy);
                    self.sync_rctx(&ret, sub_parser)?;
                    ret.map_err(Into::into)
                }
            }

            impl<'a, 'inv, Set, Ser: cote::ServicesValExt, Policy> #major_helper_ty<'a, cote::Parser<'inv, Set, Ser>, Policy> {
                pub fn set_default_rctx(&mut self) -> Result<&mut Self, cote::CoteError> {
                    self.set_rctx(cote::RunningCtx::default())
                }

                pub fn rctx(&self) -> Result<&cote::RunningCtx, cote::CoteError> {
                    self.inner_parser()
                        .rctx()
                }

                pub fn rctx_mut(&mut self) -> Result<&mut cote::RunningCtx, cote::CoteError> {
                    self.inner_parser_mut()
                        .rctx_mut()
                }

                pub fn set_rctx(&mut self, ctx: cote::RunningCtx) -> Result<&mut Self, cote::CoteError> {
                    self.inner_parser_mut()
                        .set_rctx(ctx);
                    Ok(self)
                }

                pub fn take_rctx(&mut self) -> Result<cote::RunningCtx, cote::CoteError> {
                    Ok(std::mem::take(self.rctx_mut()?))
                }
            }

            impl<'a, 'inv, Set, Ser, Policy> #major_helper_ty<'a, cote::Parser<'inv, Set, Ser>, Policy>
            where
                Ser: cote::ServicesValExt + Default,
                cote::SetCfg<Set>: cote::Config + cote::ConfigValue + Default,
                Set: cote::Set + cote::OptParser + cote::OptValidator + cote::SetValueFindExt + Default {
                pub fn display_help(&self) -> Result<(), cote::CoteError> {
                    self.display_help_with(Self::display_ctx())
                }

                pub fn display_help_with(&self, context: cote::HelpDisplayCtx) -> Result<(), cote::CoteError> {
                    let name = context.generate_name();
                    let optset = self.inner_parser().optset();

                    cote::display_help!(
                        optset,
                        &name, context.head(), context.foot(), context.width(), context.usagew()
                    ).map_err(|e| cote::raise_error!("Can not display help message: {:?}", e))
                }

                pub fn display_sub_help(&mut self, names: &[String]) -> Result<(), cote::CoteError> {
                    self.display_sub_help_idx(names, 0)
                }

                pub fn display_sub_help_idx(&mut self, names: &[String], idx: usize) -> Result<(), cote::CoteError> {
                    let len = names.len();
                    let inner_parser = self.inner_parser_mut();

                    if len >= 1 {
                        let name_matched = &names[idx] == inner_parser.name();

                        if name_matched {
                            if idx == len - 1 && len == 1 {
                                let context = Self::display_ctx();
                                let optset = inner_parser.optset();
                                // display current help message
                                return cote::display_help!(
                                    optset,
                                    &names[idx], context.head(), context.foot(), context.width(), context.usagew()
                                );
                            }
                            else if idx < len - 1 && name_matched {
                                #call_for_sub_help
                            }
                        }
                    }
                    Err(cote::CoteError::raise_error(format!("Can not display help message of names: {:?}", names)))
                }
            }
        })
    }
}

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
        abort! {
            field,
            "can not have both `sub` and `arg` configuration on same field"
        }
    } else {
        Ok(has_sub_cfg)
    }
}

pub fn gen_ret_default_policy_ty(policy_name: &str) -> Option<TokenStream> {
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
        _ => None,
    }
}

pub fn gen_policy_ty_generics(policy_name: &str) -> Option<TokenStream> {
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
        _ => None,
    }
}

pub fn gen_ret_policy_ty_generics(policy_name: &str) -> Option<TokenStream> {
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
        _ => None,
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
        abort! {
            ty, "Cote not support reference type"
        }
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
    abort! {
        ty,
        "`sub` configuration only support `Option<T>`"
    }
}

// pub fn is_option_ty(ty: &Type) -> bool {
//     if let Type::Path(path) = ty {
//         if let Some(segment) = path.path.segments.last() {
//             let ident_str = segment.ident.to_string();

//             if ident_str == "Option" {
//                 if let PathArguments::AngleBracketed(_) = &segment.arguments {
//                     return true;
//                 }
//             }
//         }
//     }
//     false
// }

pub fn gen_subapp_without_option(ty: &Type) -> syn::Result<&Ident> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return Ok(&segment.ident);
        }
    }
    abort! {
        ty,
        "can not generate sub app type"
    }
}
