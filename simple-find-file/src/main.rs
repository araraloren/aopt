use std::os::windows::prelude::MetadataExt;
use std::path::Path;

use getopt_rs::err::create_error;
use getopt_rs::tools::initialize_creator;
use getopt_rs::{getopt, prelude::*};
use regex::Regex;

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut set = SimpleSet::default();
    let mut parser = DelayParser::<UidGenerator>::default();

    initialize_creator(&mut set);
    set.add_prefix(String::from("-"));
    set.add_prefix(String::from("--"));
    set.add_prefix(String::from("+"));

    if let Ok(mut commit) = set.add_opt("directory=p@0") {
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
    for (opt, alias_prefix, alias_name, filter_type) in [
        (
            "--dir=b",
            String::from("-"),
            String::from("d"),
            FilterType::Dir,
        ),
        (
            "--link=b",
            String::from("-"),
            String::from("l"),
            FilterType::Link,
        ),
        (
            "--file=b",
            String::from("-"),
            String::from("f"),
            FilterType::File,
        ),
    ] {
        if let Ok(mut commit) = set.add_opt(opt) {
            commit.add_alias(alias_prefix, alias_name);
            let id = commit.commit()?;
            parser.add_callback(
                id,
                simple_opt_mut_cb!(move |_, set_cb, value| {
                    let ret = filte_file(set_cb, "directory", &filter_type)
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
    if let Ok(mut commit) = set.add_opt("--size=u") {
        commit.add_alias(String::from("-"), String::from("s"));
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_opt_mut_cb!(move |_, set_cb, value| {
                let filter_type = FilterType::Size(*value.as_uint().unwrap_or(&u64::MAX));
                let ret = filte_file(set_cb, "directory", &filter_type)
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
    if let Ok(mut commit) = set.add_opt("--regex=s") {
        commit.add_alias(String::from("-"), String::from("r"));
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_opt_mut_cb!(move |_, set_cb, value| {
                let filter_type =
                    FilterType::Regex(value.as_str().unwrap_or(&format!(".")).clone());
                let ret = filte_file(set_cb, "directory", &filter_type)
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
    if let Ok(mut commit) = set.add_opt("main=m") {
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_main_cb!(|_, set, _, value| {
                for file in filte_file(set, "directory", &FilterType::All) {
                    println!("{}", file);
                }
                Ok(Some(value))
            }),
        );
    }

    getopt!(&mut std::env::args().skip(1), set, parser)?;

    Ok(())
}

fn filte_file<'a>(set: &'a dyn Set, opt: &str, filter_type: &FilterType) -> Vec<&'a str> {
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
