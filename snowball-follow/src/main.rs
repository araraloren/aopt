use std::path::Path;
use std::{env::Args, io::Stdout};

use getopt_rs::err::create_error;
use getopt_rs::prelude::*;
use getopt_rs::Ustr;
use getopt_rs::{
    getopt,
    prelude::{Result, SimpleParser},
    set::{OptionSet, Set, SimpleSet},
    tools::{initialize_creator, initialize_prefix},
    uid::UidGenerator,
};
use getopt_rs_help::printer::Printer;
use getopt_rs_help::{
    store::{OptStore, PosStore},
    AppHelp,
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;
    let set = parser_command_line(std::env::args())?;

    if ! print_help(&set)? {
        if let Ok(Some(id)) = set.find("stock_id") {
            if id.has_value() {
                dbg!(id);
            }
        }
        if let Ok(Some(file)) = set.find("stock_file_list") {
            if file.has_value() {
                dbg!(file);
            }
        }
    }
    Ok(())
}

fn parser_command_line(args: Args) -> Result<SimpleSet> {
    let mut set = SimpleSet::default();
    let mut parser = SimpleParser::<UidGenerator>::default();

    initialize_creator(&mut set);
    initialize_prefix(&mut set);

    for (optstr, alias, help) in [
        ("-d=b", "--debug", "Print debug message"),
        ("-h=b", "--help", "Print help message"),
        ("-s=i", "--start", "Set start parameter of request"),
        ("-c=i", "--count", "Set count parameter of request"),
    ] {
        if let Ok(mut commit) = set.add_opt(optstr) {
            commit.add_alias(alias)?;
            commit.set_help(help);
            commit.commit()?;
        }
    }
    // process single stock id
    if let Ok(mut commit) = set.add_opt("stock_id=p@1") {
        commit.set_help("Get follow from single stock id");
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_pos_cb!(|_, _, id, _, value| {
                let mut ret = Ok(Some(value));

                for char in id.chars() {
                    if !char.is_ascii_digit() {
                        ret = Ok(None);
                    }
                }
                ret
            }),
        );
    }
    // process single stock id
    if let Ok(mut commit) = set.add_opt("stock_file_list=p@1") {
        commit.set_help("Get follow from stock list in file");
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_pos_cb!(|_, _, file, _, value| {
                let mut ret = Ok(Some(value));

                if !Path::new(file).is_file() {
                    ret = Ok(None);
                }
                ret
            }),
        );
    }
    getopt!(&mut args.skip(1), set, parser)?;

    Ok(set)
}

fn print_help(set: &dyn Set) -> Result<bool> {
    let mut is_need_help = false;

    if let Ok(Some(opt)) = set.find("help") {
        if opt.get_value().as_bool() == Some(&true) {
            is_need_help = true;
        }
    }
    if is_need_help {
        let mut app_help = simple_help_generate(set);

        app_help
            .print_cmd_help(None)
            .map_err(|e| create_error(format!("can not write help to stdout: {:?}", e)))?;
    }
    Ok(is_need_help)
}

fn simple_help_generate(set: &dyn Set) -> AppHelp<Stdout> {
    let mut help = AppHelp::default();

    help.set_name("snowball".into());

    let global = help.store.get_global_mut();

    for opt in set.iter() {
        if opt.match_style(getopt_rs::opt::Style::Pos) {
            global.add_pos(PosStore::new(
                opt.get_name(),
                opt.get_hint(),
                opt.get_help(),
                opt.get_index().unwrap().to_string().into(),
                opt.get_optional(),
            ));
        } else if !opt.match_style(getopt_rs::opt::Style::Main) {
            global.add_opt(OptStore::new(
                opt.get_name(),
                opt.get_hint(),
                opt.get_help(),
                opt.get_optional(),
            ));
        }
    }

    global.set_header(Ustr::from(
        "Get the follow people number in https://xueqiu.com/",
    ));
    global.set_footer(Ustr::from("Create by araraloren, V0.1.0"));

    help
}
