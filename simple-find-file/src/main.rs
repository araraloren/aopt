use std::io::Stdout;
use std::os::windows::prelude::MetadataExt;
use std::path::Path;

use getopt_rs::err::create_error;
use getopt_rs::tools::initialize_creator;
use getopt_rs::Ustr;
use getopt_rs::{getopt, prelude::*};
use getopt_rs_help::printer::Printer;
use getopt_rs_help::store::{OptStore, PosStore};
use getopt_rs_help::AppHelp;
use regex::Regex;

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut set = SimpleSet::default();
    let mut parser = DelayParser::<UidGenerator>::default();

    initialize_creator(&mut set);
    set.add_prefix(Ustr::from("-"));
    set.add_prefix(Ustr::from("--"));
    set.add_prefix(Ustr::from("+"));

    if let Ok(mut commit) = set.add_opt("directory=p@0") {
        commit.set_help("Set the target directory");
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_pos_cb!(|_, _, dir, _, _| {
                if !dir.is_empty() {
                    if let Ok(files) = find_file_in_directory(dir) {
                        Ok(Some(OptValue::from(files)))
                    } else {
                        Err(create_error(format!("Directory access error: {:?}", dir)))
                    }
                } else {
                    Err(create_error(format!("Directory can not be empty!")))
                }
            }),
        );
    }
    for (opt, help, alias_prefix, alias_name, mut filter_type) in [
        (
            "--dir=b",
            "Show the files type are directory",
            "-",
            "d",
            FilterType::Dir,
        ),
        (
            "--link=b",
            "Show the files type are symbol link",
            "-",
            "l",
            FilterType::Link,
        ),
        (
            "--file=b",
            "Show the files type are normal file",
            "-",
            "f",
            FilterType::File,
        ),
        (
            "--size=u",
            "Show the files size large than given size",
            "-",
            "s",
            FilterType::Size(0),
        ),
        (
            "--regex=s",
            "Show the files which name matched given regex",
            "-",
            "r",
            FilterType::Regex(String::default()),
        ),
    ] {
        if let Ok(mut commit) = set.add_opt(opt) {
            commit.set_help(help);
            commit.add_alias(&format!("{}{}", alias_prefix, alias_name));
            let id = commit.commit()?;
            parser.add_callback(
                id,
                simple_opt_mut_cb!(move |_, set_cb, value| {
                    let filter_type = filter_type.copy_value_from(&value);
                    let ret = filter_file(set_cb, "directory", &filter_type)
                        .iter()
                        .map(|&v| String::from(v))
                        .collect::<Vec<String>>();
                    if let Ok(mut filter) = set_cb.filter_mut("directory") {
                        if let Some(dir_opt) = filter.find() {
                            dir_opt.set_value(OptValue::from(ret))
                        }
                    }
                    Ok(Some(value))
                }),
            )
        }
    }
    if let Ok(mut commit) = set.add_opt("--help=b") {
        commit.add_alias("-h");
        commit.set_help("Show the help message");
        commit.commit()?;
    }
    if let Ok(mut commit) = set.add_opt("main=m") {
        commit.set_help("Main function");
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_main_cb!(|_, set, _, value| {
                let mut is_need_help = false;

                if let Some(help_value) = set.filter("--help")?.find() {
                    if help_value.get_value().as_bool() == Some(&true) {
                        is_need_help = true;
                    }
                }
                if is_need_help {
                    let mut app_help = simple_help_generate(set);

                    app_help.print_cmd_help(None).map_err(|e| {
                        create_error(format!("can not write help to stdout: {:?}", e))
                    })?;
                } else {
                    for file in filter_file(set, "directory", &FilterType::All) {
                        println!("{}", file);
                    }
                }
                Ok(Some(value))
            }),
        );
    }

    getopt!(&mut std::env::args().skip(1), set, parser)?;

    Ok(())
}

fn filter_file<'a>(set: &'a dyn Set, opt: &str, filter_type: &FilterType) -> Vec<&'a str> {
    let mut ret = vec![];
    if let Ok(filter) = set.filter(opt) {
        if let Some(dir_opt) = filter.find() {
            let value = dir_opt.get_value();
            if let Some(files) = value.as_slice() {
                for file in files {
                    if filter_type.filter(file) {
                        ret.push(file.as_str());
                    }
                }
            }
        }
    }
    ret
}

fn find_file_in_directory(dir: &str) -> color_eyre::Result<Vec<String>> {
    let mut ret = vec![];
    for entry in Path::new(dir).read_dir()? {
        let entry = entry?;

        if let Some(path) = entry.path().to_str() {
            ret.push(path.to_owned());
        }
    }
    Ok(ret)
}

#[derive(Debug, Clone)]
enum FilterType {
    All,
    Dir,
    Link,
    File,
    Size(u64),
    Regex(String),
}

impl Default for FilterType {
    fn default() -> Self {
        Self::All
    }
}

impl FilterType {
    pub fn copy_value_from(&mut self, value: &OptValue) -> &mut Self {
        match self {
            FilterType::Regex(regex_str) => {
                *regex_str = value.as_str().unwrap_or(&String::from(".")).clone();
            }
            FilterType::Size(size) => {
                *size = *value.as_uint().unwrap_or(&u64::MAX);
            }
            _ => {}
        }
        self
    }

    pub fn filter(&self, path: &str) -> bool {
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            match self {
                FilterType::All => {
                    return true;
                }
                FilterType::Dir => meta.is_dir(),
                FilterType::Link => meta.file_type().is_symlink(),
                FilterType::File => meta.is_file(),
                FilterType::Size(size) => meta.file_size() >= *size,
                FilterType::Regex(regex_str) => {
                    if let Ok(regex) = Regex::new(regex_str) {
                        regex.is_match(path)
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }
}

fn simple_help_generate(set: &dyn Set) -> AppHelp<Stdout> {
    let mut help = AppHelp::default();

    help.set_name("simple-find-file".into());

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
        "Search the given directory, show the file match the filter conditions",
    ));
    global.set_footer(Ustr::from("Create by araraloren, V0.2.0"));

    help
}
