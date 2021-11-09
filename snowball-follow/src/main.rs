use std::fmt::format;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
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

const STOCK_NUMBER_LEN: usize = 6;
const STOCK_SHANGHAI: &'static str = "SH";
const STOCK_SHENZHEN: &'static str = "SZ";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;
    let set = parser_command_line(std::env::args())?;

    if !print_help(&set)? {
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
    if let Ok(mut commit) = set.add_opt("stock_id=p@0") {
        commit.set_help("Get follow from single stock id");
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_pos_mut_cb!(|_, set, id, _, value| {
                let mut ret = Ok(None);

                if check_stock_number(id) {
                    if let Ok(Some(opt)) = set.find_mut("stock_id") {
                        let value_mut = opt.get_value_mut();

                        if value_mut.is_null() {
                            *value_mut = OptValue::from(vec![normalize_stock_number(id.as_ref())]);
                        } else {
                            if let Some(vec_mut) = value_mut.as_vec_mut() {
                                vec_mut.push(normalize_stock_number(id.as_ref()));
                            } else {
                                ret = Err(create_error(format!(
                                    "can not get vec mut ref from value"
                                )));
                            }
                        }
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
            simple_pos_mut_cb!(|_, set, file, _, value| {
                let mut ret = Ok(None);
                let fh = Path::new(file);
                if fh.is_file() {
                    if let Ok(Some(opt)) = set.find_mut("stock_id") {
                        let value_mut = opt.get_value_mut();

                        if value_mut.is_null() {
                            *value_mut = OptValue::from(vec![]);
                        }
                        if let Some(vec_mut) = value_mut.as_vec_mut() {
                            let fh = File::open(fh).map_err(|e| {
                                create_error(format!("can not read file {}: {:?}", file, e))
                            })?;
                            let mut reader = BufReader::new(fh);
                            let mut line = String::default();

                            loop {
                                if let Ok(count) = reader.read_line(&mut line) {
                                    if count > 0 {
                                        if check_stock_number(line.as_ref()) {
                                            vec_mut.push(normalize_stock_number(line.as_ref()));
                                        } else {
                                            eprintln!("`{}` is not a valid stock number", line);
                                        }
                                    } else {
                                        break;
                                    }
                                } else {
                                    ret = Err(create_error(format!(
                                        "can not read line from file {}",
                                        file
                                    )));
                                }
                            }
                        }
                    }
                }
                ret
            }),
        );
    }
    getopt!(&mut args.skip(1), set, parser)?;

    Ok(set)
}

fn check_stock_number(number: &str) -> bool {
    if number.len() > 0 && number.len() <= 6 {
        for char in number.chars() {
            if !char.is_ascii_digit() {
                return false;
            }
        }
        true
    } else {
        false
    }
}

fn normalize_stock_number(number: &str) -> String {
    let mut ret = format!("{}{}", "0".repeat(STOCK_NUMBER_LEN - number.len()), number);

    if let Some(header) = ret.get(0..2) {
        match header {
            "68" | "60" => {
                ret = format!("{}{}", STOCK_SHANGHAI, ret);
            }
            "00" | "30" => {
                ret = format!("{}{}", STOCK_SHENZHEN, ret);
            }
            _ => {
                panic!("{} is not a valid stock number", number);
            }
        }
    }
    ret
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
