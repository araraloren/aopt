use std::marker::PhantomData;

use getopt_rs::opt::value;
use getopt_rs::prelude::*;
use getopt_rs::err::report_an_error;

fn main() -> Result<()> {
    let mut set = SimpleSet::default();
    let mut parser = PreParser::<SimpleSet, UidGenerator>::default();

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

    getopt_rs::tools::initialize_log().unwrap();

    if let Ok(mut commit) = set.add_opt("cpp=c") {
        commit.set_help("run in cpp mode".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::Main(Box::new(SimpleMainCallback::new(
                |_id, set, _, value| {
                    let mut ret = Ok(Some(value));
                    if let Some(std) = set.filter("std").unwrap().find() {
                        if let Some(std) = std.get_value().as_str() {
                            if !check_compiler_std(std, "cpp") {
                                ret = report_an_error(format!(
                                    "Unsupport standard version for c++: {}",
                                    std
                                ));
                            }
                        }
                    }
                    ret
                },
            ))),
        );
    }
    if let Ok(mut commit) = set.add_opt("c=c") {
        commit.set_help("run in c mode".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::Main(Box::new(SimpleMainCallback::new(
                |_id, set, _, value| {
                    let std = set
                        .filter("std")
                        .unwrap()
                        .find()
                        .unwrap()
                        .get_value()
                        .as_str()
                        .unwrap();

                    if !check_compiler_std(std, "c") {
                        report_an_error(format!("Unsupport standard version for c++: {}", std))
                    } else {
                        Ok(Some(value))
                    }
                },
            ))),
        );
    }
    if let Ok(mut commit) = set.add_opt("-S=b") {
        commit.set_help("pass -S to compiler.".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("-E=b") {
        commit.set_help("pass -E to compiler.".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("+D=a") {
        commit.set_help("pass -D<value> to compiler.".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::Opt(Box::new(SimpleOptCallback::new(
                |id, set, value| {
                    println!(
                        "user want define a macro {:?}",
                        set.get_opt(id)
                            .unwrap()
                            .get_value()
                            .as_slice()
                            .unwrap_or(&[])
                            .last()
                    );
                    Ok(Some(value))
                },
            ))),
        );
    }
    if let Ok(mut commit) = set.add_opt("+l=a") {
        commit.set_help("pass -l<value> to compiler.".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::Opt(Box::new(SimpleOptCallback::new(
                |id, set, value| {
                    println!(
                        "user want link the library {:?}",
                        set.get_opt(id)
                            .unwrap()
                            .get_value()
                            .as_slice()
                            .unwrap()
                            .last()
                    );
                    Ok(Some(value))
                },
            ))),
        );
    }
    if let Ok(mut commit) = set.add_opt("+i=a") {
        commit.set_help("add include header to code.".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::Opt(Box::new(SimpleOptCallback::new(
                |id, set, value| {
                    println!(
                        "user want include header {:?}",
                        set.get_opt(id)
                            .unwrap()
                            .get_value()
                            .as_slice()
                            .unwrap()
                            .last()
                    );
                    Ok(Some(value))
                },
            ))),
        );
    }
    if let Ok(mut commit) = set.add_opt("+L=a") {
        commit.set_help("pass -L<value> to compiler.".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::Opt(Box::new(SimpleOptCallback::new(
                |id, set, value| {
                    println!(
                        "user want add search library search path {:?}",
                        set.get_opt(id)
                            .unwrap()
                            .get_value()
                            .as_slice()
                            .unwrap()
                            .last()
                    );
                    Ok(Some(value))
                },
            ))),
        );
    }
    if let Ok(mut commit) = set.add_opt("+I=a") {
        commit.set_help("pass -I<value> to compiler.".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::Opt(Box::new(SimpleOptCallback::new(
                |id, set, value| {
                    println!(
                        "user want add search header search path {:?}",
                        set.get_opt(id)
                            .unwrap()
                            .get_value()
                            .as_slice()
                            .unwrap()
                            .last()
                    );
                    Ok(Some(value))
                },
            ))),
        );
    }
    if let Ok(mut commit) = set.add_opt("-w=b") {
        commit.set_help("pass -Wall -Wextra -Werror to compiler.".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("-std=s") {
        commit.set_help("pass -std=<value> to compiler.".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("temp=p@*") {
        commit.set_help("pass all the arguments after '--' to compiler.".to_string());
        commit.set_name("--".to_string());
        let id = commit.commit().unwrap();
        parser.add_callback(
            id,
            OptCallback::PosMut(Box::new(SimplePosMutCallback::new(
                |id, set, arg, _index, value| {
                    // collect the arguments after --
                    let mut value = std::mem::take(set.get_opt_mut(id).unwrap().get_value_mut());
                    let ret = Ok(if value.is_vec() {
                        value.as_vec_mut().unwrap().push(arg.clone());
                        Some(value)
                    } else if (arg == "--") && (!value.is_vec()) {
                        value = OptValue::from(vec![]);
                        Some(value)
                    } else {
                        Some(value)
                    });
                    ret
                },
            ))),
        );
    }
    let mut args = &mut ["c", "a", "ops"].iter().map(|&v| String::from(v));

    let ret = parser.parse(set, &mut std::env::args().skip(1)).unwrap();

    if let Some(ret) = ret {
        dbg!(ret);
    }

    Ok(())
}

fn check_compiler_std(std: &str, compiler: &str) -> bool {
    let cpp_std: Vec<&str> = vec![
        "c++98", "c++03", "c++11", "c++0x", "c++14", "c++1y", "c++17", "c++1z", "c++20", "c++2a",
    ];
    let c_std: Vec<&str> = vec!["c89", "c90", "c99", "c11", "c17", "c2x"];
    match compiler {
        "c" => c_std.contains(&std),
        "cpp" => cpp_std.contains(&std),
        _ => false,
    }
}