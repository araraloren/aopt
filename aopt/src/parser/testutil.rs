#![allow(dead_code)]

use std::fmt::Debug;

use crate::{err::Result, prelude::*, set::Commit};

macro_rules! simple_opt_tweak {
    () => {
        Box::new(|parser, uid, checker| {
            let mut checker = checker;

            parser.add_callback(
                uid,
                simple_opt_mut_cb!(move |uid, set, value| {
                    let opt = set[uid].as_mut();

                    if let Some(checker) = checker.take() {
                        checker.checking(opt, &value);
                    }

                    Ok(Some(value))
                }),
            );
            Ok(())
        })
    };
}

macro_rules! simple_main_tweak {
    () => {
        Box::new(|parser, uid, checker| {
            let mut checker = checker;

            parser.add_callback(
                uid,
                simple_main_mut_cb!(move |uid, set, _, value| {
                    let opt = set[uid].as_mut();

                    if let Some(checker) = checker.take() {
                        checker.checking(opt, &value);
                    }

                    Ok(Some(value))
                }),
            );
            Ok(())
        })
    };
}

macro_rules! simple_pos_tweak {
    () => {
        Box::new(|parser, uid, checker| {
            let mut checker = checker;

            parser.add_callback(
                uid,
                simple_pos_mut_cb!(move |uid, set, arg, _, value| {
                    let opt = set[uid].as_mut();

                    if let Some(checker) = checker.take() {
                        checker.checking(opt, &value);
                    }

                    Ok(Some(OptValue::from(arg)))
                }),
            );
            Ok(())
        })
    };
}

macro_rules! simple_delay_pos_tweak {
    () => {
        Box::new(|parser, uid, checker| {
            let mut checker = checker;

            parser.add_callback(
                uid,
                simple_pos_mut_cb!(move |uid, set, arg, _, value| {
                    let opt = set[uid].as_mut();

                    if let Some(checker) = checker.take() {
                        checker.checking(opt, &value);
                    }
                    for opt_name in arg.chars() {
                        if let Some(modify_opt) = set.find_mut(&format!("{}", opt_name))? {
                            modify_opt.set_callback_ret(Some(OptValue::from(true)))?;
                        }
                    }

                    Ok(Some(OptValue::from(arg)))
                }),
            );
            Ok(())
        })
    };
}

pub struct TestingCase<P: Parser> {
    pub opt: &'static str,

    pub value: Option<OptValue>,

    pub commit: Option<Box<dyn FnMut(&mut Commit) -> Result<()>>>,

    pub callback: Option<Box<dyn FnMut(&mut P, Uid, Option<OptChecker>) -> Result<()>>>,

    pub checker: Option<OptChecker>,
}

impl<P: Parser> Default for TestingCase<P> {
    fn default() -> Self {
        Self {
            opt: "",
            value: None,
            commit: None,
            callback: Some(simple_opt_tweak!()),
            checker: None,
        }
    }
}

impl<P: Parser> TestingCase<P> {
    pub fn new(opt: &'static str) -> Self {
        Self {
            opt,
            ..Self::default()
        }
    }

    pub fn with_opt(mut self, opt: &'static str) -> Self {
        self.opt = opt;
        self
    }

    pub fn with_value(mut self, value: OptValue) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_commit(mut self, commit: Box<dyn FnMut(&mut Commit) -> Result<()>>) -> Self {
        self.commit = Some(commit);
        self
    }

    pub fn with_callback(
        mut self,
        callback: Box<dyn FnMut(&mut P, Uid, Option<OptChecker>) -> Result<()>>,
    ) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn with_checker(mut self, checker: OptChecker) -> Self {
        self.checker = Some(checker);
        self
    }

    pub fn with_default_opt_tweak(mut self) -> Self {
        self.callback = Some(simple_opt_tweak!());
        self
    }

    pub fn with_default_main_tweak(mut self) -> Self {
        self.callback = Some(simple_main_tweak!());
        self
    }

    pub fn with_default_pos_tweak(mut self) -> Self {
        self.callback = Some(simple_pos_tweak!());
        self
    }

    pub fn with_default_delay_pos_tweak(mut self) -> Self {
        self.callback = Some(simple_delay_pos_tweak!());
        self
    }

    pub fn set_opt(&mut self, opt: &'static str) {
        self.opt = opt;
    }

    pub fn set_value(&mut self, value: OptValue) {
        self.value = Some(value);
    }

    pub fn set_commit(&mut self, commit: Box<dyn FnMut(&mut Commit) -> Result<()>>) {
        self.commit = Some(commit);
    }

    pub fn set_callback(
        &mut self,
        callback: Box<dyn FnMut(&mut P, Uid, Option<OptChecker>) -> Result<()>>,
    ) {
        self.callback = Some(callback);
    }

    pub fn set_checker(&mut self, checker: OptChecker) {
        self.checker = Some(checker);
    }

    pub fn get_opt(&self) -> &&'static str {
        &self.opt
    }

    pub fn get_value(&self) -> Option<&OptValue> {
        self.value.as_ref()
    }

    pub fn get_commit(&self) -> Option<&Box<dyn FnMut(&mut Commit) -> Result<()>>> {
        self.commit.as_ref()
    }

    pub fn get_callback(
        &self,
    ) -> Option<&Box<dyn FnMut(&mut P, Uid, Option<OptChecker>) -> Result<()>>> {
        self.callback.as_ref()
    }

    pub fn get_checker(&self) -> Option<&OptChecker> {
        self.checker.as_ref()
    }

    pub fn add_test(&mut self, set: &mut dyn Set, parser: &mut P) -> Result<()> {
        let mut commit = set.add_opt(self.opt)?;

        if let Some(tweak) = self.commit.as_mut() {
            tweak.as_mut()(&mut commit)?;
        }

        let uid = commit.commit()?;

        if let Some(tweak) = self.callback.as_mut() {
            tweak.as_mut()(parser, uid, self.checker.take())?;
        }

        Ok(())
    }

    pub fn check_ret(&mut self, set: &mut dyn Set) -> Result<()> {
        if let Some(value) = &self.value {
            if let Some(real_ret) = set.get_value(self.opt)? {
                assert_eq!(value, real_ret);
            }
        }
        Ok(())
    }
}

pub struct Checker(Box<dyn Fn(&OptChecker, &mut dyn Opt)>);

impl Debug for Checker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checker").finish()
    }
}

impl Checker {
    pub fn new(func: Box<dyn Fn(&OptChecker, &mut dyn Opt)>) -> Self {
        Self(func)
    }

    pub fn call(&self, opt_checker: &OptChecker, opt: &mut dyn Opt) {
        self.0(opt_checker, opt);
    }
}

#[derive(Debug)]
pub struct OptChecker {
    pub type_name: &'static str,

    pub prefix: &'static str,

    pub name: &'static str,

    pub optional: Option<bool>,

    pub index: Option<OptIndex>,

    pub deactivate_style: Option<bool>,

    pub callback_value: Option<OptValue>,

    pub default_value: Option<OptValue>,

    pub alias: Option<Vec<(&'static str, &'static str)>>,

    pub checker: Option<Checker>,
}

impl Default for OptChecker {
    fn default() -> Self {
        Self {
            type_name: "",
            prefix: "",
            name: "",
            optional: None,
            index: None,
            deactivate_style: None,
            callback_value: None,
            default_value: None,
            alias: None,
            checker: None,
        }
    }
}

impl OptChecker {
    pub fn new(type_name: &'static str, prefix: &'static str, name: &'static str) -> Self {
        Self {
            type_name,
            prefix,
            name,
            ..Self::default()
        }
    }

    pub fn with_type_name(mut self, type_name: &'static str) -> Self {
        self.type_name = type_name;
        self
    }

    pub fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = Some(optional);
        self
    }

    pub fn with_index(mut self, index: OptIndex) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_deactivate_style(mut self, deactivate_style: bool) -> Self {
        self.deactivate_style = Some(deactivate_style);
        self
    }

    pub fn with_callback_value(mut self, callback_value: OptValue) -> Self {
        self.callback_value = Some(callback_value);
        self
    }

    pub fn with_default_value(mut self, default_value: OptValue) -> Self {
        self.default_value = Some(default_value);
        self
    }

    pub fn with_alias(mut self, alias: Vec<(&'static str, &'static str)>) -> Self {
        self.alias = Some(alias);
        self
    }

    pub fn with_checker(mut self, checker: Checker) -> Self {
        self.checker = Some(checker);
        self
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn get_prefix(&self) -> &'static str {
        self.prefix
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_optional(&self) -> Option<&bool> {
        self.optional.as_ref()
    }

    pub fn get_index(&self) -> Option<&OptIndex> {
        self.index.as_ref()
    }

    pub fn get_deactivate_style(&self) -> Option<&bool> {
        self.deactivate_style.as_ref()
    }

    pub fn get_callback_value(&self) -> Option<&OptValue> {
        self.callback_value.as_ref()
    }

    pub fn get_default_value(&self) -> Option<&OptValue> {
        self.default_value.as_ref()
    }

    pub fn get_alias(&self) -> Option<&Vec<(&'static str, &'static str)>> {
        self.alias.as_ref()
    }

    pub fn get_checker(&self) -> Option<&Checker> {
        self.checker.as_ref()
    }

    pub fn set_type_name(&mut self, type_name: &'static str) {
        self.type_name = type_name;
    }

    pub fn set_prefix(&mut self, prefix: &'static str) {
        self.prefix = prefix;
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = name;
    }

    pub fn set_optional(&mut self, optional: bool) {
        self.optional = Some(optional);
    }

    pub fn set_index(&mut self, index: OptIndex) {
        self.index = Some(index);
    }

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) {
        self.deactivate_style = Some(deactivate_style);
    }

    pub fn set_callback_value(&mut self, callback_value: OptValue) {
        self.callback_value = Some(callback_value);
    }

    pub fn set_default_value(&mut self, default_value: OptValue) {
        self.default_value = Some(default_value);
    }

    pub fn set_alias(&mut self, alias: Vec<(&'static str, &'static str)>) {
        self.alias = Some(alias);
    }

    pub fn set_checker(&mut self, checker: Checker) {
        self.checker = Some(checker);
    }

    pub fn checking(&self, opt: &mut dyn Opt, cb_value: &OptValue) {
        assert_eq!(opt.get_name().as_str(), self.name);
        assert_eq!(opt.get_type_name().as_str(), self.type_name);
        assert_eq!(opt.get_prefix().as_str(), self.prefix);
        if let Some(value) = self.optional {
            assert_eq!(value, opt.get_optional());
        }
        assert_eq!(self.index.as_ref(), opt.get_index());
        if let Some(value) = self.deactivate_style {
            assert_eq!(value, opt.is_deactivate_style());
        }
        //println!("compare {:?} <=> {:?}", &cb_value, &self.callback_value);
        if let Some(value) = &self.callback_value {
            if value.is_vec() && cb_value.is_vec() {
                if let Some(testing_values) = value.as_vec() {
                    if let Some(cb_values) = cb_value.as_vec() {
                        for value in cb_values {
                            assert!(testing_values.contains(value));
                        }
                    }
                }
            } else if value.is_vec() {
                if let Some(testing_values) = value.as_vec() {
                    if let Some(cb_values) = cb_value.as_str() {
                        assert!(testing_values.contains(cb_values));
                    }
                }
            } else {
                assert!(value.eq(cb_value));
            }
        }
        if let Some(value) = &self.default_value {
            assert_eq!(value, opt.get_default_value());
        }
        if let Some(value) = &self.alias {
            for (prefix, name) in value {
                assert!(opt.match_alias(gstr(prefix), gstr(name)));
            }
        }
        if let Some(checker) = self.checker.as_ref() {
            checker.call(&self, opt);
        }
    }
}

pub fn nonopt_testcases<P: Parser>() -> Vec<TestingCase<P>> {
    vec![
        TestingCase {
            opt: "n=m",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_type_name("m")
                    .with_name("n")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            ..TestingCase::default()
        }
        .with_default_main_tweak(),
        TestingCase {
            opt: "o=m!",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("m")
                    .with_name("o")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            ..TestingCase::default()
        }
        .with_default_main_tweak(),
        TestingCase {
            opt: "p=c",
            checker: Some(
                OptChecker::default()
                    .with_optional(false)
                    .with_type_name("c")
                    .with_name("p")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::Forward(1)),
            ),
            value: Some(OptValue::from(true)),
            ..TestingCase::default()
        }
        .with_default_main_tweak(),
        TestingCase {
            opt: "q=p@2",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("p")
                    .with_name("q")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::Forward(2)),
            ),
            value: Some(OptValue::from("pos2")),
            ..TestingCase::default()
        }
        .with_default_pos_tweak(),
        TestingCase {
            opt: "r=p@*",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("p")
                    .with_name("r")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::AnyWhere),
            ),
            value: Some(OptValue::from("pos6")),
            ..TestingCase::default()
        }
        .with_default_pos_tweak(),
        TestingCase {
            opt: "s=p@-2",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("p")
                    .with_name("s")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::Backward(2)),
            ),
            value: Some(OptValue::from("pos5")),
            ..TestingCase::default()
        }
        .with_default_pos_tweak(),
        TestingCase {
            opt: "t=p@[3,5]",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("p")
                    .with_name("t")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::List(vec![3, 5])),
            ),
            value: Some(OptValue::from("pos5")),
            ..TestingCase::default()
        }
        .with_default_pos_tweak(),
        TestingCase {
            opt: "u=p@-[1,3,5,6]",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("p")
                    .with_name("u")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::Except(vec![1, 3, 5, 6])),
            ),
            value: Some(OptValue::from("pos4")),
            ..TestingCase::default()
        }
        .with_default_pos_tweak(),
        TestingCase {
            opt: "v=p@>4",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("p")
                    .with_name("v")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::Greater(4)),
            ),
            value: Some(OptValue::from("pos6")),
            ..TestingCase::default()
        }
        .with_default_pos_tweak(),
        TestingCase {
            opt: "w=p@<3",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("")
                    .with_type_name("p")
                    .with_name("w")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true))
                    .with_index(OptIndex::Less(3)),
            ),
            value: Some(OptValue::from("pos2")),
            ..TestingCase::default()
        }
        .with_default_pos_tweak(),
    ]
}

pub fn long_prefix_opt_testcases<P: Parser>() -> Vec<TestingCase<P>> {
    vec![
        TestingCase {
            opt: "--aopt=a",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("--")
                    .with_type_name("a")
                    .with_name("aopt")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(vec![
                        "value5".to_owned(),
                        "value6".to_owned(),
                    ]))
                    .with_alias(vec![("--", "aopt-alias1"), ("--", "aopt-alias2")]),
            ),
            commit: Some(Box::new(|commit: &mut Commit| -> Result<()> {
                commit.add_alias("--aopt-alias1")?;
                commit.add_alias("--aopt-alias2")?;
                Ok(())
            })),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "--bopt=b",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("--")
                    .with_type_name("b")
                    .with_name("bopt")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "--copt=i",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("--")
                    .with_type_name("i")
                    .with_name("copt")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(42i64))
                    .with_alias(vec![("--", "copt-alias1"), ("--", "copt-alias2")]),
            ),
            commit: Some(Box::new(|commit: &mut Commit| -> Result<()> {
                commit.add_alias("--copt-alias1")?;
                commit.add_alias("--copt-alias2")?;
                Ok(())
            })),
            value: Some(OptValue::from(42i64)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "--dopt=f",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("--")
                    .with_type_name("f")
                    .with_name("dopt")
                    .with_deactivate_style(false),
            ),
            commit: Some(Box::new(|commit: &mut Commit| -> Result<()> {
                commit.set_default_value(OptValue::from(3.14f64));
                Ok(())
            })),
            value: Some(OptValue::from(3.14f64)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "--eopt=s",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("--")
                    .with_type_name("s")
                    .with_name("eopt")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from("value7")),
            ),
            value: Some(OptValue::from("value7")),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "--fopt=u",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("--")
                    .with_type_name("u")
                    .with_name("fopt")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(988u64)),
            ),
            value: Some(OptValue::from(988u64)),
            ..TestingCase::default()
        },
    ]
}

pub fn shorting_prefix_opt_testcases<P: Parser>() -> Vec<TestingCase<P>> {
    vec![
        TestingCase {
            opt: "-a=a",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("a")
                    .with_name("a")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(vec![
                        "value1".to_owned(),
                        "value2".to_owned(),
                    ])),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-b=a!",
            checker: Some(
                OptChecker::default()
                    .with_optional(false)
                    .with_prefix("-")
                    .with_type_name("a")
                    .with_name("b")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(vec![
                        "value3".to_owned(),
                        "value4".to_owned(),
                    ])),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-c=b",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("c")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-d=b!",
            checker: Some(
                OptChecker::default()
                    .with_optional(false)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("d")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-e=b/",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("e")
                    .with_deactivate_style(true)
                    .with_callback_value(OptValue::from(false)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-f=f",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("f")
                    .with_name("f")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(3.1415926)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-g=f!",
            checker: Some(
                OptChecker::default()
                    .with_optional(false)
                    .with_prefix("-")
                    .with_type_name("f")
                    .with_name("g")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(2.718281)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-h=i",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("i")
                    .with_name("h")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(42i64)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-i=i!",
            checker: Some(
                OptChecker::default()
                    .with_optional(false)
                    .with_prefix("-")
                    .with_type_name("i")
                    .with_name("i")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(-100000i64)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-j=s",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("s")
                    .with_name("j")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from("foo")),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-k=s!",
            checker: Some(
                OptChecker::default()
                    .with_optional(false)
                    .with_prefix("-")
                    .with_type_name("s")
                    .with_name("k")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from("bar")),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-l=u",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("u")
                    .with_name("l")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(1988u64)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-m=u!",
            checker: Some(
                OptChecker::default()
                    .with_optional(false)
                    .with_prefix("-")
                    .with_type_name("u")
                    .with_name("m")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(2202u64)),
            ),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-x=i",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("i")
                    .with_name("x")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(42i64))
                    .with_default_value(OptValue::from(4200i64)),
            ),
            commit: Some(Box::new(|commit: &mut Commit| -> Result<()> {
                commit.set_default_value(OptValue::from(4200i64));
                Ok(())
            })),
            value: Some(OptValue::from(4200i64)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-y=i",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("i")
                    .with_name("y")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(42i64)),
            ),
            commit: Some(Box::new(|commit: &mut Commit| -> Result<()> {
                commit.add_alias("--yopt")?;
                commit.add_alias("--yopt-int")?;
                Ok(())
            })),
            value: Some(OptValue::from(42i64)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-z=b/",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("z")
                    .with_deactivate_style(true)
                    .with_callback_value(OptValue::from(false)),
            ),
            value: Some(OptValue::from(false)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-1=b/",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("1")
                    .with_deactivate_style(true)
                    .with_callback_value(OptValue::from(false)),
            ),
            value: Some(OptValue::from(false)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-2=b/",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("2")
                    .with_deactivate_style(true)
                    .with_callback_value(OptValue::from(false)),
            ),
            value: Some(OptValue::from(false)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-4=b",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("4")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            value: Some(OptValue::from(true)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-5=b",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("5")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            value: Some(OptValue::from(true)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-6=b",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("6")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            value: Some(OptValue::from(true)),
            ..TestingCase::default()
        },
        TestingCase {
            opt: "-?=b",
            checker: Some(
                OptChecker::default()
                    .with_optional(true)
                    .with_prefix("-")
                    .with_type_name("b")
                    .with_name("?")
                    .with_deactivate_style(false)
                    .with_callback_value(OptValue::from(true)),
            ),
            value: Some(OptValue::from(true)),
            ..TestingCase::default()
        },
    ]
}
