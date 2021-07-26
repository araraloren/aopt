use std::marker::PhantomData;

use getopt_rs::opt::value;
use getopt_rs::prelude::*;
use getopt_rs::set::Commit;

fn main() -> Result<()> {
    // let mut set = SimpleSet::default();
    // let mut parser = SimpleParser::<SimpleSet, UidGenerator>::default();

    // set.add_creator(Box::new(IntCreator::default()));
    // set.add_creator(Box::new(BoolCreator::default()));
    // set.add_creator(Box::new(StrCreator::default()));
    // set.add_creator(Box::new(FltCreator::default()));
    // set.add_creator(Box::new(UintCreator::default()));
    // set.add_creator(Box::new(ArrayCreator::default()));
    // set.add_creator(Box::new(CmdCreator::default()));
    // set.add_creator(Box::new(PosCreator::default()));
    // set.add_creator(Box::new(MainCreator::default()));
    // set.add_prefix(String::from("-"));
    // set.add_prefix(String::from("--"));
    // set.add_prefix(String::from("+"));

    // getopt_rs::tools::initialize_log().unwrap();

    // if let Ok(mut commit) = set.add_opt("cpp=c") {
    //     commit.set_help("run in cpp mode".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::Main(Box::new(callback::SimpleMainCallback::new(
    //             |_id, set, _, value| {
    //                 let mut ret = Ok(Some(value));
    //                 if let Some(std) = set.filter("std").unwrap().find() {
    //                     if let Some(std) = std.get_value().as_str() {
    //                         if !check_compiler_std(std, "cpp") {
    //                             ret = report_an_error(format!(
    //                                 "Unsupport standard version for c++: {}",
    //                                 std
    //                             ));
    //                         }
    //                     }
    //                 }
    //                 ret
    //             },
    //         ))),
    //     );
    // }
    // if let Ok(mut commit) = set.add_opt("c=c") {
    //     commit.set_help("run in c mode".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::Main(Box::new(callback::SimpleMainCallback::new(
    //             |_id, set, _, value| {
    //                 let std = set
    //                     .filter("std")
    //                     .unwrap()
    //                     .find()
    //                     .unwrap()
    //                     .get_value()
    //                     .as_str()
    //                     .unwrap();

    //                 if !check_compiler_std(std, "c") {
    //                     report_an_error(format!("Unsupport standard version for c++: {}", std))
    //                 } else {
    //                     Ok(Some(value))
    //                 }
    //             },
    //         ))),
    //     );
    // }
    // if let Ok(mut commit) = set.add_opt("-S=b") {
    //     commit.set_help("pass -S to compiler.".to_string());
    //     commit.commit().unwrap();
    // }
    // if let Ok(mut commit) = set.add_opt("-E=b") {
    //     commit.set_help("pass -E to compiler.".to_string());
    //     commit.commit().unwrap();
    // }
    // if let Ok(mut commit) = set.add_opt("+D=a") {
    //     commit.set_help("pass -D<value> to compiler.".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::Opt(Box::new(callback::SimpleOptCallback::new(
    //             |id, set, value| {
    //                 println!(
    //                     "user want define a macro {:?}",
    //                     set.get_opt(id)
    //                         .unwrap()
    //                         .get_value()
    //                         .as_slice()
    //                         .unwrap()
    //                         .last()
    //                 );
    //                 Ok(Some(value))
    //             },
    //         ))),
    //     );
    // }
    // if let Ok(mut commit) = set.add_opt("+l=a") {
    //     commit.set_help("pass -l<value> to compiler.".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::Opt(Box::new(callback::SimpleOptCallback::new(
    //             |id, set, value| {
    //                 println!(
    //                     "user want link the library {:?}",
    //                     set.get_opt(id)
    //                         .unwrap()
    //                         .get_value()
    //                         .as_slice()
    //                         .unwrap()
    //                         .last()
    //                 );
    //                 Ok(Some(value))
    //             },
    //         ))),
    //     );
    // }
    // if let Ok(mut commit) = set.add_opt("+i=a") {
    //     commit.set_help("add include header to code.".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::Opt(Box::new(callback::SimpleOptCallback::new(
    //             |id, set, value| {
    //                 println!(
    //                     "user want include header {:?}",
    //                     set.get_opt(id)
    //                         .unwrap()
    //                         .get_value()
    //                         .as_slice()
    //                         .unwrap()
    //                         .last()
    //                 );
    //                 Ok(Some(value))
    //             },
    //         ))),
    //     );
    // }
    // if let Ok(mut commit) = set.add_opt("+L=a") {
    //     commit.set_help("pass -L<value> to compiler.".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::Opt(Box::new(callback::SimpleOptCallback::new(
    //             |id, set, value| {
    //                 println!(
    //                     "user want add search library search path {:?}",
    //                     set.get_opt(id)
    //                         .unwrap()
    //                         .get_value()
    //                         .as_slice()
    //                         .unwrap()
    //                         .last()
    //                 );
    //                 Ok(Some(value))
    //             },
    //         ))),
    //     );
    // }
    // if let Ok(mut commit) = set.add_opt("+I=a") {
    //     commit.set_help("pass -I<value> to compiler.".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::Opt(Box::new(callback::SimpleOptCallback::new(
    //             |id, set, value| {
    //                 println!(
    //                     "user want add search header search path {:?}",
    //                     set.get_opt(id)
    //                         .unwrap()
    //                         .get_value()
    //                         .as_slice()
    //                         .unwrap()
    //                         .last()
    //                 );
    //                 Ok(Some(value))
    //             },
    //         ))),
    //     );
    // }
    // if let Ok(mut commit) = set.add_opt("-w=b") {
    //     commit.set_help("pass -Wall -Wextra -Werror to compiler.".to_string());
    //     commit.commit().unwrap();
    // }
    // if let Ok(mut commit) = set.add_opt("-std=s") {
    //     commit.set_help("pass -std=<value> to compiler.".to_string());
    //     commit.commit().unwrap();
    // }
    // if let Ok(mut commit) = set.add_opt("temp=p@*") {
    //     commit.set_help("pass all the arguments after '--' to compiler.".to_string());
    //     commit.set_name("--".to_string());
    //     let id = commit.commit().unwrap();
    //     parser.add_callback(
    //         id,
    //         OptCallback::PosMut(Box::new(callback::SimplePosMutCallback::new(
    //             |id, set, arg, _index, value| {
    //                 // collect the arguments after --
    //                 let mut value = std::mem::take(set.get_opt_mut(id).unwrap().get_value_mut());
    //                 let ret = Ok(if value.is_vec() {
    //                     value.as_vec_mut().unwrap().push(arg.clone());
    //                     Some(value)
    //                 } else if (arg == "--") && (!value.is_vec()) {
    //                     value = OptValue::from(vec![]);
    //                     Some(value)
    //                 } else {
    //                     Some(value)
    //                 });
    //                 ret
    //             },
    //         ))),
    //     );
    // }
    // let mut args = &mut ["c", "a", "ops"].iter().map(|&v| String::from(v));

    // let ret = parser.parse(set, &mut std::env::args().skip(1)).unwrap();

    // if let Some(ret) = ret {
    //     dbg!(ret);
    // }

    #[derive(Debug)]
    struct DataChecker {
        type_name: &'static str,

        deactivate_style: bool,

        cb_value: OptValue,

        default_value: OptValue,

        name: &'static str,

        prefix: &'static str,

        alias: Vec<(&'static str, &'static str)>,

        optional: bool,

        index: Option<OptIndex>,
    }

    impl Default for DataChecker {
        fn default() -> Self {
            Self {
                type_name: "",
                deactivate_style: false,
                cb_value: OptValue::default(),
                default_value: OptValue::default(),
                name: "",
                prefix: "",
                alias: vec![],
                optional: true,
                index: None,
            }
        }
    }

    impl DataChecker {
        pub fn check(&self, opt: &dyn Opt, cb_value: &OptValue) {
            println!("checking {:?} and {:?} <-> {:?}", opt, cb_value, self);
            assert_eq!(opt.get_name(), self.name);
            assert_eq!(opt.is_need_invoke(), true);
            assert_eq!(opt.get_optional(), self.optional);
            assert!(self.default_value.eq(opt.get_default_value()));
            assert_eq!(opt.get_type_name(), self.type_name);
            assert_eq!(self.deactivate_style, opt.is_deactivate_style());
            assert_eq!(self.prefix, opt.get_prefix());
            assert_eq!(opt.get_index(), self.index.as_ref());
            for (prefix, name) in &self.alias {
                assert!(opt.match_alias(prefix, name));
            }
            if self.cb_value.is_vec() && cb_value.is_vec() {
                if let Some(testing_values) = self.cb_value.as_vec() {
                    if let Some(values) = cb_value.as_vec() {
                        for value in values {
                            assert!(testing_values.contains(value));
                        }
                    }
                }
            } else {
                assert!(self.cb_value.eq(cb_value));
            }
        }
    }

    struct TestingCase<S: Set, P: Parser<S>> {
        opt_str: &'static str,

        ret_value: Option<OptValue>,

        commit_tweak: Option<Box<dyn FnMut(&mut Commit)>>,

        callback_tweak: Option<Box<dyn FnMut(&mut P, Uid, Option<DataChecker>)>>,

        checker: Option<DataChecker>,

        marker: PhantomData<S>,
    }

    impl<S: Set, P: Parser<S>> TestingCase<S, P> {
        pub fn do_test(&mut self, set: &mut S, parser: &mut P) -> Result<()> {
            let mut commit = set.add_opt(self.opt_str)?;

            if let Some(tweak) = self.commit_tweak.as_mut() {
                tweak.as_mut()(&mut commit);
            }
            let uid = commit.commit()?;

            if let Some(tweak) = self.callback_tweak.as_mut() {
                tweak.as_mut()(parser, uid, self.checker.take());
            }
            Ok(())
        }

        pub fn check_ret(&mut self, set: &mut S) -> Result<()> {
            if let Some(ret_value) = self.ret_value.as_ref() {
                if let Some(opt) = set.filter(self.opt_str)?.find() {
                    assert!(ret_value.eq(opt.as_ref().get_value()));
                }
            }
            Ok(())
        }
    }

    macro_rules! simple_cb_tweak {
        () => {
            Some(Box::new(
                |parser: &mut SimpleParser<SimpleSet, UidGenerator>,
                 uid,
                 checker: Option<DataChecker>| {
                    let mut checker = checker;

                    parser.add_callback(
                        uid,
                        simple_opt_cb!(move |uid, set, value| {
                            let opt = set[uid].as_ref();

                            if let Some(checker) = checker.take() {
                                checker.check(opt, &value);
                            }
                            Ok(Some(value))
                        }),
                    );
                },
            ))
        };
    }

    let mut set = SimpleSet::new();
    let mut parser = SimpleParser::new(UidGenerator::default());

    set.add_prefix("+".to_owned());

    let testing_cases = &mut [
        TestingCase {
            opt_str: "-a=i",
            ret_value: Some(OptValue::from(42i64)),
            commit_tweak: None,
            callback_tweak: simple_cb_tweak!(),
            checker: Some(DataChecker {
                type_name: "i",
                cb_value: OptValue::from(42i64),
                name: "a",
                prefix: "-",
                ..DataChecker::default()
            }),
            marker: PhantomData::default(),
        },
        TestingCase {
            opt_str: "-b=u",
            ret_value: Some(OptValue::from(42u64)),
            commit_tweak: None,
            callback_tweak: simple_cb_tweak!(),
            checker: Some(DataChecker {
                type_name: "u",
                cb_value: OptValue::from(42u64),
                name: "b",
                prefix: "-",
                ..DataChecker::default()
            }),
            marker: PhantomData::default(),
        },
        TestingCase {
            opt_str: "-c=s",
            ret_value: Some(OptValue::from("string")),
            commit_tweak: None,
            callback_tweak: simple_cb_tweak!(),
            checker: Some(DataChecker {
                type_name: "s",
                cb_value: OptValue::from("string"),
                name: "c",
                prefix: "-",
                ..DataChecker::default()
            }),
            marker: PhantomData::default(),
        },
        TestingCase {
            opt_str: "-d=f",
            ret_value: Some(OptValue::from(3.1415926f64)),
            commit_tweak: None,
            callback_tweak: simple_cb_tweak!(),
            checker: Some(DataChecker {
                type_name: "f",
                cb_value: OptValue::from(3.1415926f64),
                name: "d",
                prefix: "-",
                ..DataChecker::default()
            }),
            marker: PhantomData::default(),
        },
        TestingCase {
            opt_str: "-e=b",
            ret_value: Some(OptValue::from(true)),
            commit_tweak: None,
            callback_tweak: simple_cb_tweak!(),
            checker: Some(DataChecker {
                type_name: "b",
                cb_value: OptValue::from(true),
                name: "e",
                prefix: "-",
                ..DataChecker::default()
            }),
            marker: PhantomData::default(),
        },
        TestingCase {
            opt_str: "-f=a",
            ret_value: Some(OptValue::from(vec!["lucy".to_owned(), "lily".to_owned()])),
            commit_tweak: None,
            callback_tweak: simple_cb_tweak!(),
            checker: Some(DataChecker {
                type_name: "a",
                cb_value: OptValue::from(vec!["lucy".to_owned(), "lily".to_owned()]),
                name: "f",
                prefix: "-",
                ..DataChecker::default()
            }),
            marker: PhantomData::default(),
        },
        TestingCase {
            opt_str: "-g=i",
            ret_value: Some(OptValue::from(42i64)),
            commit_tweak: Some(Box::new(|commit: &mut Commit| {
                commit.add_alias("+".to_owned(), "g-i64".to_owned());
            })),
            callback_tweak: simple_cb_tweak!(),
            checker: Some(DataChecker {
                type_name: "i",
                cb_value: OptValue::from(42i64),
                name: "g",
                prefix: "-",
                alias: vec![("+", "g-i64")],
                ..DataChecker::default()
            }),
            marker: PhantomData::default(),
        },
        TestingCase {
            opt_str: "-h=i",
            ret_value: None,
            commit_tweak: None,
            callback_tweak: simple_cb_tweak!(),
            checker: None,
            marker: PhantomData::default(),
        },
    ];

    for testing_case in testing_cases.iter_mut() {
        testing_case.do_test(&mut set, &mut parser)?;
    }

    let input = &mut [
        "-a",
        "42",
        "-b=42",
        "-cstring",
        "-f",
        "lucy",
        "-d",
        "3.1415926",
        "-e",
        "-f",
        "lily",
        "+g-i64",
        "42",
    ]
    .iter()
    .map(|&v| String::from(v));

    let ret = parser.parse(set, input)?;

    if let Some(mut ret) = ret {
        for testing_case in testing_cases.iter_mut() {
            testing_case.check_ret(&mut ret.set)?;
        }
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
