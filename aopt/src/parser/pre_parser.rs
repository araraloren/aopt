use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::DerefMut;

use super::HashMapIter;
use super::OptValueKeeper;
use super::Parser;
use super::ParserState;

use crate::arg::Argument;
use crate::err::Result;
use crate::opt::OptCallback;
use crate::opt::OptValue;
use crate::opt::Style;
use crate::proc::Info;
use crate::proc::Matcher;
use crate::proc::NonOptMatcher;
use crate::proc::OptMatcher;
use crate::proc::Proc;
use crate::set::OptionInfo;
use crate::set::Set;
use crate::uid::Generator;
use crate::uid::Uid;
use crate::Ustr;

#[derive(Debug, Default)]
pub struct PreParser<G>
where
    G: Generator + Debug + Default,
{
    uid_gen: G,

    subscriber_info: Vec<Box<dyn Info>>,

    callback: HashMap<Uid, RefCell<OptCallback>>,

    noa: Vec<Ustr>,
}

impl<G> PreParser<G>
where
    G: Generator + Debug + Default,
{
    pub fn new(uid_gen: G) -> Self {
        Self {
            uid_gen,
            ..Self::default()
        }
    }
}

impl<G> Parser for PreParser<G>
where
    G: Generator + Debug + Default,
{
    fn parse(
        &mut self,
        set: &mut dyn Set,
        iter: &mut dyn Iterator<Item = Argument>,
    ) -> Result<bool> {
        let set = set;

        // copy the prefix, so we don't need borrow set
        let prefix: Vec<Ustr> = set.get_prefix().iter().map(|v| v.clone()).collect();

        // add info to Proc
        for opt in set.opt_iter() {
            self.subscriber_info
                .push(Box::new(OptionInfo::from(opt.get_uid())));
        }

        // reset set and do pre check
        info!("reset and do pre check");
        set.reset();
        self.pre_check(set)?;

        let parser_state = vec![
            ParserState::PSEqualWithValue,
            ParserState::PSArgument,
            ParserState::PSBoolean,
            ParserState::PSMultipleOption,
            ParserState::PSEmbeddedValue,
        ];

        // iterate the Arguments, generate option context
        // send it to Publisher
        info!("start process option ...");
        while let Some(mut arg) = iter.next() {
            let mut matched = false;
            let mut consume = false;

            debug!(?arg, "iterator Argument ...");
            if let Ok(ret) = arg.parse(&prefix) {
                if ret {
                    debug!(?arg, "after parsing ...");
                    for gen_style in &parser_state {
                        if let Some(ret) = gen_style.gen_opt::<OptMatcher>(&arg)? {
                            let mut proc = ret;

                            if self.process(&mut proc, set)? {
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
                debug!("!!! {:?} not matching, will add it to noa", &arg);
                if let Some(noa) = &arg.current {
                    self.noa.push(noa.clone());
                }
            }
        }

        trace!(?self.noa, "current non-option argument");

        // do option check
        self.check_opt(set)?;

        let noa_count = self.noa.len();

        if noa_count > 0 {
            let gen_style = ParserState::PSNonCmd;

            info!("start process {:?} ...", &gen_style);
            if let Some(ret) =
                gen_style.gen_nonopt::<NonOptMatcher>(&self.noa[0], noa_count as u64, 1)?
            {
                let mut proc = ret;

                self.process(&mut proc, set)?;
            }

            let gen_style = ParserState::PSNonPos;

            info!("start process {:?} ...", &gen_style);
            for index in 1..=noa_count {
                if let Some(ret) = gen_style.gen_nonopt::<NonOptMatcher>(
                    &self.noa[index - 1],
                    noa_count as u64,
                    index as u64,
                )? {
                    let mut proc = ret;

                    self.process(&mut proc, set)?;
                }
            }
        }

        // check pos and cmd
        self.check_nonopt(set)?;

        let gen_style = ParserState::PSNonMain;

        info!("start process {:?} ...", &gen_style);
        if let Some(ret) =
            gen_style.gen_nonopt::<NonOptMatcher>(&Ustr::default(), noa_count as u64, 1)?
        {
            let mut proc = ret;

            self.process(&mut proc, set)?;
        }

        // do post check
        self.post_check(set)?;

        Ok(true)
    }

    fn invoke_callback(
        &self,
        uid: Uid,
        set: &mut dyn Set,
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
                OptCallback::Main(cb) => {
                    let noaref: Vec<&str> = self.noa.iter().map(|v| v.as_ref()).collect();
                    cb.as_mut().call(uid, set, &noaref, value)
                }
                OptCallback::MainMut(cb) => {
                    let noaref: Vec<&str> = self.noa.iter().map(|v| v.as_ref()).collect();
                    cb.as_mut().call(uid, set, &noaref, value)
                }
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

    fn get_noa(&self) -> &[Ustr] {
        &self.noa
    }

    fn reset(&mut self) {
        self.uid_gen.reset();
        self.noa.clear();
        self.subscriber_info.clear();
    }
}

impl<G> Proc<NonOptMatcher> for PreParser<G>
where
    G: Generator + Debug + Default,
{
    fn process(&mut self, msg: &mut NonOptMatcher, set: &mut dyn Set) -> Result<bool> {
        let matcher = msg;
        let mut matched = false;

        debug!(?matcher, "NonOptMatcher got message");
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set).unwrap_or(None);

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
                            // make matched true, if any of NonOpt callback return Some(*)
                            value = self.invoke_callback(uid, set, noa_index, value.unwrap())?;
                            if value.is_some() {
                                matched = true;
                            }
                        } else {
                            // if a Cmd is matched, then the M matched
                            if opt.match_style(Style::Cmd) {
                                matched = true;
                            }
                        }
                        // reborrow the opt avoid the compiler error
                        // reset the matcher, we need match all the NonOpt
                        debug!(?value, "get callback return value");
                        set[uid].as_mut().set_invoke(false);
                        matcher.reset();
                    }

                    // set the value after invoke
                    set[uid].as_mut().set_callback_ret(value)?;
                }
            }
        }
        if !matched {
            matcher.undo();
        }
        Ok(matched)
    }
}

impl<G> Proc<OptMatcher> for PreParser<G>
where
    G: Generator + Debug + Default,
{
    fn process(&mut self, msg: &mut OptMatcher, set: &mut dyn Set) -> Result<bool> {
        let matcher = msg;
        let mut value_keeper = HashMap::<Uid, Vec<OptValueKeeper>>::new();

        debug!(?matcher, "OptMatcher got message");
        for info in self.subscriber_info.iter() {
            let uid = info.info_uid();
            let ctx = matcher.process(uid, set).unwrap_or(None);

            if let Some(ctx) = ctx {
                let opt = set[uid].as_mut();

                if let Some(noa_index) = ctx.get_matched_index() {
                    let invoke_callback = opt.is_need_invoke();
                    let value = ctx.take_value();

                    assert_eq!(value.is_some(), true);
                    if invoke_callback {
                        opt.set_invoke(false);
                    }

                    // add the value to value keeper, call the callback after cmd/pos processed
                    info!("add {:?} to delay parser value keeper", &uid);
                    value_keeper
                        .entry(uid)
                        .or_insert(vec![])
                        .push(OptValueKeeper {
                            noa_index,
                            value: value.unwrap(),
                        });
                }
            }
        }
        if matcher.is_matched() {
            // do value set and invoke callback
            let uids: Vec<_> = value_keeper.keys().map(|v| v.clone()).collect();

            for uid in uids {
                if let Some((_, values)) = value_keeper.remove_entry(&uid) {
                    for value in values {
                        let mut parsed_value = Some(value.value);
                        let has_callback = self.get_callback(uid).is_some();

                        if has_callback {
                            // invoke callback of current option
                            parsed_value = self.invoke_callback(
                                uid,
                                set,
                                value.noa_index,
                                parsed_value.unwrap(),
                            )?;
                        }

                        set[uid].as_mut().set_callback_ret(parsed_value)?;
                    }
                }
            }
        } else {
            matcher.undo();
        }
        Ok(matcher.is_matched())
    }
}

#[cfg(test)]
mod test {
    use crate::err::Result;
    use crate::getopt;
    use crate::parser::testutil::*;
    use crate::prelude::*;

    #[test]
    fn testing_simple_parser() {
        assert!(do_simple_test().is_ok());
    }

    fn do_simple_test() -> Result<()> {
        let mut testing_cases = vec![];

        testing_cases.append(&mut nonopt_testcases());
        testing_cases.append(&mut long_prefix_opt_testcases());
        testing_cases.append(&mut shorting_prefix_opt_testcases());

        let mut set = SimpleSet::default()
            .with_default_prefix()
            .with_default_creator();
        let mut parser = PreParser::<UidGenerator>::default();

        for testing_case in testing_cases.iter_mut() {
            if testing_case.value == Some(OptValue::from("pos6")) {
                testing_case.set_value(OptValue::from("--unknow-opt"));
            }
            testing_case.add_test(&mut set, &mut parser)?;
        }

        let mut args = [
            "p",
            "-a=value1",
            "-a",
            "value2",
            "-bvalue3",
            "-b",
            "value4",
            "-c",
            "-d",
            "-/e",
            "-f3.1415926",
            "-g=2.718281",
            "-h",
            "42",
            "-i-100000",
            "-j",
            "foo",
            "-k=bar",
            "-l1988",
            "-m=2202",
            "-/z12",
            "-456",
            "-?",
            "pos2",
            "pos3",
            "pos4",
            "pos5",
            "--yopt-int=42",
            "--aopt",
            "value5",
            "--aopt=value6",
            "--copt=42",
            "--eopt",
            "value7",
            "--fopt=988",
            "--unknow-opt",
        ]
        .iter()
        .map(|&v| String::from(v));

        if let Some(ret) = getopt!(&mut args, set, parser)? {
            for testing_case in testing_cases.iter_mut() {
                testing_case.check_ret(ret.1)?;
            }
        }

        assert_eq!(
            parser.get_noa(),
            &[
                Ustr::from("p"),
                Ustr::from("pos2"),
                Ustr::from("pos3"),
                Ustr::from("pos4"),
                Ustr::from("pos5"),
                Ustr::from("--unknow-opt")
            ]
        );
        Ok(())
    }
}
