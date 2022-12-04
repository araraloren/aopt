use std::path::Path;
use std::{ops::Deref, os::windows::prelude::MetadataExt};

use aopt::Error;
use aopt::{getopt, prelude::*};
use aopt_help::prelude::*;
use regex::Regex;

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut parser = AFwdParser::default();

    parser
        .add_opt("directory=p@1")?
        .set_help("Set the target directory")
        .on(|_: &mut ASet, _: &mut ASer, dir: ctx::Value<String>| {
            if !dir.is_empty() {
                if let Ok(files) = find_file_in_directory(dir.deref()) {
                    Ok(Some(files))
                } else {
                    Err(Error::raise_error(format!(
                        "Directory access error: {:?}",
                        dir
                    )))
                }
            } else {
                Err(Error::raise_error(
                    "Directory can not be empty!".to_string(),
                ))
            }
        })?;

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
        parser
            .add_opt(opt)?
            .set_help(help)
            .add_alias(format!("{}{}", alias_prefix, alias_name))
            .fallback(
                move |set: &mut ASet, ser: &mut ASer, val: ctx::Value<String>| {
                    String::sve_filter(set["directory"].uid(), ser, move |path: &String| {
                        let filter_type = filter_type.copy_value_from(val.take());
                        filter_type.filter(path)
                    })?;
                    Ok(None)
                },
            )?;
    }
    parser
        .add_opt("--help=b")?
        .add_alias("-h")
        .set_help("Show the help message");

    parser
        .add_opt("main=m")?
        .set_help("Main function")
        .fallback(|set: &mut ASet, ser: &mut ASer| {
            if *bool::sve_val(set["--help"].uid(), ser)? {
                let mut app_help = getopt_help!(set);

                app_help.print_cmd_help(None).map_err(|e| {
                    Error::raise_error(format!("can not write help to stdout: {:?}", e))
                })?;
            } else {
                for file in String::sve_vals(set["directory"].uid(), ser)? {
                    println!("{}", file);
                }
            }
            Ok(None)
        })?;

    getopt!(std::env::args().skip(1), &mut parser)?;

    Ok(())
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
    pub fn copy_value_from(&mut self, value: String) -> &mut Self {
        match self {
            FilterType::Regex(regex_str) => {
                *regex_str = value;
            }
            FilterType::Size(size) => {
                *size = value.parse::<u64>().unwrap_or(u64::MAX);
            }
            _ => {}
        }
        self
    }

    pub fn filter(&self, path: &str) -> bool {
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            match self {
                FilterType::All => true,
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
