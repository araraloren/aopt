use std::borrow::Cow;
use std::ops::Deref;
use std::path::Path;

use aopt::ctx::VecStore;
use aopt::Error;
use aopt::{getopt, prelude::*};
use regex::Regex;

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut parser = ADelayParser::default();

    parser
        .add_opt("directory=p!@1")?
        .set_help("The target directory will be search")
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
        })?
        .then(VecStore);

    for (opt, help, alias_prefix, alias_name, filter_type) in [
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
                move |set: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| {
                    let mut filter_type = filter_type.clone();

                    set["directory"].filter(move |path: &String| {
                        let filter_type = filter_type.copy_value_from(val.take());

                        !filter_type.filter(path)
                    })?;
                    Ok(None::<String>)
                },
            )?;
    }
    parser
        .add_opt("--help=b")?
        .add_alias("-h")
        .set_help("Show the help message")
        .on(
            |set: &mut ASet, _: &mut ASer| -> Result<Option<()>, Error> {
                display_help(set).map_err(|e| {
                    Error::raise_error(format!("can not write help to stdout: {:?}", e))
                })?;
                std::process::exit(0)
            },
        )?;

    parser
        .add_opt("main=m")?
        .set_help("Main function")
        .fallback(|set: &mut ASet, _: &mut ASer| {
            for file in set["directory"].vals::<String>()? {
                println!("{}", file);
            }
            Ok(None::<()>)
        })?;

    getopt!(Args::from_env(), &mut parser)?;

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
                FilterType::Size(size) => meta.len() >= *size,
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

fn display_help<S: Set>(set: &S) -> Result<(), aopt_help::Error> {
    use aopt_help::prelude::Block;
    use aopt_help::store::Store;

    let foot = format!(
        "Create by {} v{}",
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_VERSION")
    );
    let mut app_help = aopt_help::AppHelp::new(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_DESCRIPTION"),
        &foot,
        aopt_help::prelude::Style::default(),
        std::io::stdout(),
        50,
        50,
    );
    let global = app_help.global_mut();

    global.add_block(Block::new("option", "[OPTION]", "", "OPTION:", ""))?;
    global.add_block(Block::new("args", "[ARGS]", "", "ARGS:", ""))?;
    for opt in set.iter() {
        if opt.mat_style(Style::Pos) {
            global.add_store(
                "args",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::default(),
                    !opt.force(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Argument)
            || opt.mat_style(Style::Boolean)
            || opt.mat_style(Style::Combined)
        {
            global.add_store(
                "option",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::default(),
                    !opt.force(),
                    false,
                ),
            )?;
        }
    }

    app_help.display(true)?;

    Ok(())
}
