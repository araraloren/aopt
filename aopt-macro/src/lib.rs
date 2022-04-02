extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, Expr};

/// Parsing arguments using for `getopt!` or `getoptd!`.
///
/// Arguments should be like: `$iter`(getopt input arguments), `$($parser),+`(one or more parser)
#[derive(Debug)]
struct GetoptArgs {
    iter: Expr,
    parsers: Punctuated<Expr, Comma>,
}

impl Parse for GetoptArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parsers = Punctuated::new();
        let iter: Expr = input.parse()?;
        let _: Comma = input.parse()?;
        let first: Expr = input.parse()?;

        parsers.push(first);
        while input.peek(Comma) {
            parsers.push_punct(input.parse()?);
        }

        Ok(Self { iter, parsers })
    }
}

#[proc_macro]
pub fn getoptd(input: TokenStream) -> TokenStream {
    let getopt_args = parse_macro_input!(input as GetoptArgs);
    let iterator = &getopt_args.iter;
    let iterator = quote! { #iterator };

    let mut getopt_init = quote! {
        let mut parsers = vec![];
    };

    getopt_init.extend(getopt_args.parsers.iter().map(|p| match p {
        Expr::Path(path) => {
            quote! {
                parsers.push(&mut #path);
            }
        }
        Expr::Reference(reference) => {
            if reference.mutability.is_some() {
                quote! {
                    quote! {
                        parsers.push(#reference);
                    }
                }
            } else {
                syn::Error::new_spanned(reference, "need an instance or a mutable reference")
                    .to_compile_error()
            }
        }
        expr => {
            quote! {
                parsers.push(#expr);
            }
        }
    }));

    let ret = quote! {{
        #getopt_init
        getopt_dynparser(#iterator, parsers)
    }};
    ret.into()
}

#[proc_macro]
pub fn getopt(input: TokenStream) -> TokenStream {
    let getopt_args = parse_macro_input!(input as GetoptArgs);
    let iterator = &getopt_args.iter;
    let iterator = quote! { #iterator };

    let mut getopt_init = quote! {
        let mut parsers = vec![];
    };

    getopt_init.extend(getopt_args.parsers.iter().map(|p| match p {
        Expr::Path(path) => {
            quote! {
                parsers.push(&mut #path);
            }
        }
        Expr::Reference(reference) => {
            if reference.mutability.is_some() {
                quote! {
                    parsers.push(#reference);
                }
            } else {
                syn::Error::new_spanned(reference, "need an instance or a mutable reference")
                    .to_compile_error()
            }
        }
        expr => {
            quote! {
                parsers.push(#expr);
            }
        }
    }));

    let ret = quote! {{
        #getopt_init
        getopt_parser(#iterator, parsers)
    }};
    ret.into()
}

/// Parsing arguments using for `getopt_help!`.
///
/// Arguments should be like: `$set`(option set), `$($cmd_name),*`(zero or more cmd name)
#[derive(Debug)]
struct HelpArgs {
    set: Expr,
    cmds: Punctuated<Expr, Comma>,
}

impl Parse for HelpArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cmds = Punctuated::new();
        let set: Expr = input.parse()?;

        if input.peek(Comma) && input.parse::<Comma>().is_ok() {
            while !input.is_empty() {
                cmds.push(input.parse()?);
                if input.peek(Comma) {
                    cmds.push_punct(input.parse()?);
                } else {
                    break;
                }
            }
        }

        Ok(Self { set, cmds })
    }
}

#[proc_macro]
pub fn getopt_help(input: TokenStream) -> TokenStream {
    let help_args = parse_macro_input!(input as HelpArgs);
    let set = help_args.set;
    let cmds = help_args.cmds;

    let mut help_code = quote! {};

    if cmds.is_empty() {
        help_code.extend(quote! {{
            let mut help = AppHelp::<std::io::Stdout, DefaultFormat>::default();
            help.set_name(gstr(env!("CARGO_PKG_NAME")));
            let global = help.store.get_global_mut();
            for opt in #set.opt_iter() {
                if opt.match_style(aopt::opt::Style::Pos) {
                    global.add_pos(PosStore::new(
                        opt.get_name(),
                        opt.get_hint(),
                        opt.get_help(),
                        opt.get_index().unwrap().to_string().into(),
                        opt.get_optional(),
                    ));
                } else if opt.match_style(aopt::opt::Style::Argument)
                    || opt.match_style(aopt::opt::Style::Boolean)
                    || opt.match_style(aopt::opt::Style::Multiple)
                {
                    global.add_opt(OptStore::new(
                        opt.get_name(),
                        opt.get_hint(),
                        opt.get_help(),
                        opt.get_type_name(),
                        opt.get_optional(),
                    ));
                }
            }
            global.set_header(gstr(env!("CARGO_PKG_DESCRIPTION")));
            global.set_footer(gstr(&format!(
                "Create by {} v{}",
                env!("CARGO_PKG_AUTHORS"),
                env!("CARGO_PKG_VERSION")
            )));
            help
        }});
    } else {
        help_code.extend(quote! {
            let mut cmds_arg: Vec<&str> = vec![];
        });
        help_code.extend(cmds.iter().map(|p| {
            quote! {
                cmds_arg.push(#p);
            }
        }));
        help_code.extend(quote! {{
            let mut help = AppHelp::<std::io::Stdout, DefaultFormat>::default();
            help.set_name(gstr(env!("CARGO_PKG_NAME")));
            let version = gstr(&format!(
                "Create by {} v{}",
                env!("CARGO_PKG_AUTHORS"),
                env!("CARGO_PKG_VERSION")
            ));
            help.store.new_sec(gstr("default")).set_help(gstr("Commands:")).commit();
            let global = help.store.get_global_mut();
            global.set_header(gstr(env!("CARGO_PKG_DESCRIPTION")));
            global.set_footer(version.clone());
            for cmd_name in cmds_arg {
                if let Ok(Some(opt)) = #set.find(cmd_name) {
                    let mut search_cmd = help.store.new_cmd(gstr(cmd_name));
                    search_cmd
                        .set_footer(version.clone())
                        .set_hint(opt.get_hint())
                        .set_help(opt.get_help());
                    for opt in #set.opt_iter() {
                        if opt.match_style(aopt::opt::Style::Pos) {
                            search_cmd.add_pos(PosStore::new(
                                opt.get_name(),
                                opt.get_hint(),
                                opt.get_help(),
                                opt.get_index().unwrap().to_string().into(),
                                opt.get_optional(),
                            ));
                        } else if opt.match_style(aopt::opt::Style::Argument)
                            || opt.match_style(aopt::opt::Style::Boolean)
                            || opt.match_style(aopt::opt::Style::Multiple)
                        {
                            search_cmd.add_opt(OptStore::new(
                                opt.get_name(),
                                opt.get_hint(),
                                opt.get_help(),
                                opt.get_type_name(),
                                opt.get_optional(),
                            ));
                        }
                    }
                    search_cmd.commit();
                    help.store.attach_cmd(gstr("default"), gstr(cmd_name));
                }
            }
            help
        }});
    }
    let ret = quote! {{ #help_code }};

    ret.into()
}

#[derive(Debug)]
struct ParameterOrNormalArgs {
    name: Option<syn::Ident>,
    value: Expr,
}

impl Parse for ParameterOrNormalArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let value;

        if input.peek2(syn::token::Eq) {
            if input.peek(syn::Ident::peek_any) {
                name = Some(input.parse()?);
                input.parse::<syn::Token![=]>()?;
                value = input.parse()?;
            } else if input.peek(syn::Token![default]) {
                name = Some(syn::Ident::new("default", proc_macro2::Span::call_site()));
                input.parse::<syn::Token![=]>()?;
                value = input.parse()?;
            } else {
                value = input.parse()?;
            }
        } else {
            value = input.parse()?;
        }
        Ok(Self { name, value })
    }
}

/// Parsing arguments using for `opt_create!`.
///
/// Arguments should be like: `$init`(option create string), `$help?`(help message),
/// `$callback?`(callback), `$($ident = $value),*`(zero or more setting)
#[derive(Debug)]
struct CreateArgs {
    parser: Expr,
    init: Expr,
    parameter_args: Punctuated<ParameterOrNormalArgs, Comma>,
}

impl Parse for CreateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parameter_args = Punctuated::new();
        let parser: Expr = input.parse()?;
        let _: Comma = input.parse()?;
        let init: Expr = input.parse()?;

        if input.peek(Comma) && input.parse::<Comma>().is_ok() {
            while !input.is_empty() {
                let arg: ParameterOrNormalArgs = input.parse()?;

                parameter_args.push(arg);
                if input.peek(Comma) {
                    parameter_args.push_punct(input.parse()?);
                } else {
                    break;
                }
            }
        }

        Ok(Self {
            parser,
            init,
            parameter_args,
        })
    }
}

#[proc_macro]
pub fn getopt_add(input: TokenStream) -> TokenStream {
    let getopt_args = parse_macro_input!(input as CreateArgs);
    let init = getopt_args.init;
    let parser = getopt_args.parser;
    let mut found_help = false;
    let mut callback = None;
    let mut output = quote! {
        let init_string = #init.into();
        let mut create_info = CreateInfo::parse(init_string, #parser.get_prefix());
    };

    output.extend(getopt_args.parameter_args.iter().map(|v| {
        let name = &v.name;
        let expr = &v.value;

        if let Some(name) = name {
            match name.to_string().as_str() {
                "help" => {
                    quote! {{
                        let value = #expr.into();
                        create_info = create_info.and_then(|mut ci| { ci.set_help(value); Ok(ci) });
                    }}
                }
                "name" => {
                    quote! {{
                        let value = #expr.into();
                        create_info = create_info.and_then(|mut ci| { ci.set_name(value); Ok(ci) });
                    }}
                }
                "prefix" => {
                    quote! {{
                        let value = #expr.into();
                        create_info = create_info.and_then(|mut ci| { ci.set_prefix(value); Ok(ci) });
                    }}
                }
                "index" => {
                    quote! {{
                        let value = #expr.into();
                        create_info = create_info.and_then(|mut ci| { ci.set_index(value); Ok(ci) });
                    }}
                }
                "default" => {
                    quote! {{
                        let value = #expr;
                        create_info = create_info.and_then(|mut ci| { ci.set_default_value(value); Ok(ci) });
                    }}
                }
                "hint" => {
                    quote! {{
                        let value = #expr.into();
                        create_info = create_info.and_then(|mut ci| { ci.set_hint(value); Ok(ci) });
                    }}
                }
                "alias" => {
                    quote! {{
                        let value = #expr.into();
                        create_info = create_info.and_then(|mut ci| { ci.add_alias(value)?; Ok(ci) });
                    }}
                }
                "callback" => {
                    callback = Some(expr.clone());
                    quote! {}
                }
                _ => syn::Error::new_spanned(name, "Not support option field name").to_compile_error(),
            }
        }
        else {
            if !found_help {
                found_help = true;
                quote! {{
                    let value = #expr.into();
                    create_info = create_info.and_then(|mut ci| { ci.set_help(value); Ok(ci) });
                }}
            }
            else if callback.is_none() {
                callback = Some(expr.clone());
                quote! { }
            }
            else {
                syn::Error::new_spanned(syn::Ident::new("default", proc_macro2::Span::call_site()), "Not support more than three position arguments").to_compile_error()
            }
        }
    }));

    output.extend(quote! {
        let uid = create_info.and_then(|mut ci| #parser.add_opt_ci(ci));
    });

    if let Some(callback) = callback {
        output.extend(quote! {
            let uid = uid.and_then(|uid| {
                #parser.add_callback(uid, #callback);
                Ok(uid)
            });
        });
    }

    let ret = quote! {{
        #output
        uid
    }};

    ret.into()
}
