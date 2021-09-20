use std::path::Path;

use getopt_rs::err::create_error;
use getopt_rs::{getopt, prelude::*};

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut set = SimpleSet::default();
    let mut parser = DelayParser::<UidGenerator>::default();

    set.add_creator(Box::new(IntCreator::default()));
    set.add_creator(Box::new(BoolCreator::default()));
    set.add_creator(Box::new(StrCreator::default()));
    set.add_creator(Box::new(FltCreator::default()));
    set.add_creator(Box::new(UintCreator::default()));
    set.add_creator(Box::new(ArrayCreator::default()));
    set.add_creator(Box::new(CmdCreator::default()));
    set.add_creator(Box::new(PosCreator::default()));
    set.add_creator(Box::new(MainCreator::default()));
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
    if let Ok(mut commit) = set.add_opt("--dir=b") {
        commit.add_alias(String::from("-"), String::from("d"));
        let id = commit.commit()?;
        parser.add_callback(
            id,
            simple_opt_mut_cb!(|_, set, value| {
                let ret = filte_file(set, "directory", |f| Path::new(f).is_dir())
                            .iter()
                            .map(|&v| String::from(v)).collect::<Vec<String>>();
                if let Ok(mut filter) = set.filter_mut("directory") {
                    if let Some(dir_opt) = filter.find() {
                        dir_opt
                            .set_value(OptValue::from(ret))
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
                for file in filte_file(set, "directory", |_| true) {
                    println!("{}", file);
                }
                Ok(Some(value))
            }),
        );
    }

    getopt!(&mut std::env::args().skip(1), set, parser)?;

    Ok(())
}

fn filte_file<'a, F: Fn(&str) -> bool>(set: &'a dyn Set, opt: &str, filte_op: F) -> Vec<&'a str> {
    let mut ret = vec![];
    if let Ok(filter) = set.filter(opt) {
        if let Some(dir_opt) = filter.find() {
            let value = dir_opt.get_value();
            if let Some(files) = value.as_slice() {
                for file in files {
                    if filte_op(file) {
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
