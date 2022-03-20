extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, Expr};

#[derive(Debug)]
struct GetoptArgs {
    iter: Expr,
    parsers: Punctuated<Expr, Comma>,
}

impl Parse for GetoptArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parsers = Punctuated::new();
        let iter: Expr = input.parse()?;
        let comma: Comma = input.parse()?;

        while !input.is_empty() {
            let parser: Expr = input.parse()?;
            let next_comma: syn::Result<Comma> = input.parse();

            parsers.push(parser);
            match next_comma {
                Ok(is_comma) => {
                    parsers.push_punct(is_comma);
                }
                Err(_) => {
                    // last parser
                    parsers.push_punct(comma);
                }
            }
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

#[derive(Debug)]
struct HelpArgs {
    set: Expr,
    cmds: Punctuated<Expr, Comma>,
}

impl Parse for HelpArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cmds = Punctuated::new();
        let set: Expr = input.parse()?;
        let comma: syn::Result<Comma> = input.parse();

        if let Ok(comma) = comma {
            while !input.is_empty() {
                let cmd: Expr = input.parse()?;
                let next_comma: syn::Result<Comma> = input.parse();

                cmds.push(cmd);
                match next_comma {
                    Ok(is_comma) => {
                        cmds.push_punct(is_comma);
                    }
                    Err(_) => {
                        // last parser
                        cmds.push_punct(comma);
                    }
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
