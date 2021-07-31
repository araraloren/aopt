use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;

use super::HashMapIter;
use super::ParserState;
use super::{Parser, ReturnValue};
use crate::arg::ArgStream;
use crate::err::Result;
use crate::opt::{OptCallback, OptValue, Style};
use crate::proc::{Info, Matcher, NonOptMatcher, OptMatcher, Proc};
use crate::set::{OptionInfo, Set};
use crate::uid::{Generator, Uid};

#[derive(Debug, Default)]
pub struct SimpleParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    uid_gen: G,

    subscriber_info: Vec<Box<dyn Info>>,

    callback: HashMap<Uid, RefCell<OptCallback>>,

    noa: Vec<String>,

    marker: PhantomData<S>,
}

impl<S, G> SimpleParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    pub fn new(uid_gen: G) -> Self {
        Self {
            uid_gen,
            ..Self::default()
        }
    }
}

impl<S, G> Parser<S> for SimpleParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>> {
        let mut argstream = ArgStream::from(iter);
        let mut set = set;
        let mut iter = argstream.iter_mut();

        // copy the prefix, so we don't need borrow set
        let prefix: Vec<String> = set.get_prefix().iter().map(|v| v.clone()).collect();

        // add info to Proc
        for opt in set.iter() {
            self.subscriber_info
                .push(Box::new(OptionInfo::from(opt.get_uid())));
        }

        // reset set and do pre check
        set.reset();
        self.pre_check(&set)?;

        let parser_state = vec![
            ParserState::PSEqualWithValue,
            ParserState::PSArgument,
            ParserState::PSBoolean,
            ParserState::PSMultipleOption,
            ParserState::PSEmbeddedValue,
        ];

        // iterate the Arguments, generate option context
        // send it to Publisher
        debug!("Start process option ...");
        while let Some(arg) = iter.next() {
            let mut matched = false;
            let mut consume = false;

            debug!("Get next Argument => {:?}", &arg);
            if let Ok(ret) = arg.parse(&prefix) {
                if ret {
                    debug!(" ... parsed: {:?}", &arg);
                    for gen_style in &parser_state {
                        if let Some(ret) = gen_style.gen_opt::<OptMatcher>(arg) {
                            let mut proc = ret;

                            if self.process(&mut proc, &mut set)? {
                                if proc.is_matched() {
                                    matched = true;
                                }
                                if proc.is_comsume_argument() {
                                    consume = true;
                                }
                                if matched {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if matched && consume {
                iter.next();
            } else if !matched {
                debug!("!!! Not matching {:?}, add it to noa", &arg);
                if let Some(noa) = &arg.current {
                    self.noa.push(noa.clone());
                }
            }
        }

        // do option check
        self.check_opt(&set)?;

        let noa_count = self.noa.len();

        if noa_count > 0 {
            let gen_style = ParserState::PSNonCmd;

            debug!("Start process {:?} ...", &gen_style);
            if let Some(ret) =
                gen_style.gen_nonopt::<NonOptMatcher>(&self.noa[0], noa_count as u64, 1)
            {
                let mut proc = ret;

                self.process(&mut proc, &mut set)?;
            }

            let gen_style = ParserState::PSNonPos;

            debug!("Start process {:?} ...", &gen_style);
            for index in 1..=noa_count {
                if let Some(ret) = gen_style.gen_nonopt::<NonOptMatcher>(
                    &self.noa[index - 1],
                    noa_count as u64,
                    index as u64,
                ) {
                    let mut proc = ret;

                    self.process(&mut proc, &mut set)?;
                }
            }
        }

        // check pos and cmd
        self.check_nonopt(&set)?;

        let gen_style = ParserState::PSNonMain;

        debug!("Start process {:?} ...", &gen_style);
        if let Some(ret) =
            gen_style.gen_nonopt::<NonOptMatcher>(&String::new(), noa_count as u64, 1)
        {
            let mut proc = ret;

            self.process(&mut proc, &mut set)?;
        }

        // do post check
        self.post_check(&set)?;

        Ok(Some(ReturnValue {
            set: set,
            noa: self.noa.clone(),
        }))
    }

    fn invoke_callback(
        &self,
        uid: Uid,
        set: &mut S,
        noa_index: usize,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        if let Some(callback) = self.callback.get(&uid) {
            debug!("calling callback of option<{}>", uid);
            match callback.borrow_mut().deref_mut() {
                OptCallback::Opt(cb) => cb.as_mut().call(uid, set, value),
                OptCallback::OptMut(cb) => cb.as_mut().call(uid, set, value),
                OptCallback::Pos(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_index - 1], noa_index as u64, value)
                }
                OptCallback::PosMut(cb) => {
                    cb.as_mut()
                        .call(uid, set, &self.noa[noa_index - 1], noa_index as u64, value)
                }
                OptCallback::Main(cb) => cb.as_mut().call(uid, set, &self.noa, value),
                OptCallback::MainMut(cb) => cb.as_mut().call(uid, set, &self.noa, value),
                OptCallback::Null => Ok(None),
            }
        } else {
            Ok(Some(value))
        }
    }

    fn add_callback(&mut self, uid: Uid, callback: OptCallback) {
        self.callback.insert(uid, RefCell::new(callback));
    }

    fn get_callback(&self, uid: Uid) -> Option<&RefCell<OptCallback>> {
        self.callback.get(&uid)
    }

    fn callback_iter(&self) -> HashMapIter<'_, Uid, RefCell<OptCallback>> {
        self.callback.iter()
    }

    fn reset(&mut self) {
        self.uid_gen.reset();
        self.noa.clear();
        self.subscriber_info.clear();
    }
}

impl<S, G> Proc<S, NonOptMatcher> for SimpleParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    fn process(&mut self, msg: &mut NonOptMatcher, set: &mut S) -> Result<bool> {
        let matcher = msg;
        let mut matched = false;

        debug!("Got message<{}>: {:?}", &matcher.uid(), &matcher);
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set)?;

            if let Some(ctx) = ctx {
                let opt = set[uid].as_mut();

                if let Some(noa_index) = ctx.get_matched_index() {
                    let invoke_callback = opt.is_need_invoke();
                    let mut value = ctx.take_value();

                    assert_eq!(value.is_some(), true);
                    if invoke_callback {
                        let has_callback = self.get_callback(uid).is_some();

                        if has_callback {
                            // invoke callback of current option/non-option
                            value = self.invoke_callback(uid, set, noa_index, value.unwrap())?;
                            if value.is_some() {
                                // make matched true, if any of NonOpt callback return Some(*)
                                matched = true;
                            }
                        } else {
                            // if a Cmd is matched, then the M matched
                            if opt.match_style(Style::Cmd) {
                                matched = true;
                            }
                        }
                        // reborrow the opt avoid the compiler error
                        debug!("In Proc, get return value of option<{}> = {:?}", uid, value);
                        set[uid].as_mut().set_invoke(false);
                        // reset the matcher, we need match all the NonOpt
                        matcher.reset();
                    }

                    // set the value after invoke
                    set[uid].as_mut().set_callback_ret(value)?;
                }
            }
        }
        Ok(matched)
    }
}

impl<S, G> Proc<S, OptMatcher> for SimpleParser<S, G>
where
    G: Generator + Debug + Default,
    S: Set + Default,
{
    fn process(&mut self, msg: &mut OptMatcher, set: &mut S) -> Result<bool> {
        let matcher = msg;

        debug!("Got message<{}>: {:?}", &matcher.uid(), &matcher);
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set)?;

            if let Some(ctx) = ctx {
                let opt = set[uid].as_mut();

                if let Some(noa_index) = ctx.get_matched_index() {
                    let invoke_callback = opt.is_need_invoke();
                    let mut value = ctx.take_value();

                    assert_eq!(value.is_some(), true);

                    if invoke_callback {
                        // invoke callback of current option/non-option
                        value = self.invoke_callback(uid, set, noa_index, value.unwrap())?;

                        debug!("Get return value of option<{}> = {:?}", uid, value);
                        set[uid].as_mut().set_invoke(false);
                    }

                    set[uid].as_mut().set_callback_ret(value)?;
                }
            }
        }
        Ok(matcher.is_matched())
    }
}

#[cfg(test)]
mod test {
    use crate::parser::testutil::*;
    use crate::{prelude::*, set::Commit};
    use std::marker::PhantomData;

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

    #[test]
    fn testing_simple_parser() {
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

        assert!(do_simple_test(testing_cases).is_ok());
    }

    fn do_simple_test(
        testing_cases: &mut [TestingCase<SimpleSet, SimpleParser<SimpleSet, UidGenerator>>],
    ) -> Result<()> {
        let mut set = SimpleSet::new();
        let mut parser = SimpleParser::new(UidGenerator::default());

        set.add_prefix("+".to_owned());

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
}
