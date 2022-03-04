use super::ParserState;
use std::fmt::Debug;

use super::Policy;
use super::Service;
use crate::arg::Argument;
use crate::err::Result;
use crate::proc::Matcher;
use crate::proc::NonOptMatcher;
use crate::proc::OptMatcher;
use crate::set::OptionInfo;
use crate::set::Set;
use ustr::Ustr;

#[derive(Debug, Clone, Default)]
pub struct PrePolicy;

impl PrePolicy {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }
}

impl<S: Set, SS: Service<S>> Policy<S, SS> for PrePolicy {
    fn parse(
        &mut self,
        set: &mut S,
        service: &mut SS,
        iter: &mut dyn Iterator<Item = Argument>,
    ) -> Result<bool> {
        // copy the prefix, so we don't need borrow set
        let prefix: Vec<Ustr> = set.get_prefix().iter().map(|v| v.clone()).collect();

        // add info to Service
        for opt in set.opt_iter() {
            service
                .get_subscriber_info_mut()
                .push(Box::new(OptionInfo::from(opt)));
        }

        // reset set and do pre check
        info!("reset and do pre check");
        set.reset();
        service.pre_check(set)?;

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
                        if let Some(mut proc) = service.gen_opt::<OptMatcher>(&arg, &gen_style)? {
                            service.matching(&mut proc, set, true)?;

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
            if matched && consume {
                iter.next();
            } else if !matched {
                debug!("!!! {:?} not matching, will add it to noa", &arg);
                if let Some(noa) = &arg.current {
                    service.get_noa_mut().push(noa.clone());
                }
            }
        }

        let noa = service.get_noa().clone();

        trace!(?noa, "current non-option argument");
        info!("do opt check");
        service.opt_check(set)?;

        let noa_count = noa.len();

        if noa_count > 0 {
            let gen_style = ParserState::PSNonCmd;

            info!("start process {:?} ...", &gen_style);
            if let Some(mut proc) =
                service.gen_nonopt::<NonOptMatcher>(&noa[0], noa_count, 1, &gen_style)?
            {
                service.matching(&mut proc, set, true)?;
            }

            let gen_style = ParserState::PSNonPos;

            info!("start process {:?} ...", &gen_style);
            for index in 1..=noa_count {
                if let Some(mut proc) = service.gen_nonopt::<NonOptMatcher>(
                    &noa[index - 1],
                    noa_count,
                    index,
                    &gen_style,
                )? {
                    service.matching(&mut proc, set, true)?;
                }
            }
        }

        // check pos and cmd
        info!("do nonopt check");
        service.nonopt_check(set)?;

        let gen_style = ParserState::PSNonMain;

        info!("start process {:?} ...", &gen_style);
        if let Some(mut proc) =
            service.gen_nonopt::<NonOptMatcher>(&Ustr::default(), noa_count, 1, &gen_style)?
        {
            service.matching(&mut proc, set, true)?;
        }

        // do post check
        service.post_check(set)?;

        Ok(true)
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

        let mut parser = PreParser::default();

        for testing_case in testing_cases.iter_mut() {
            if testing_case.value == Some(OptValue::from("pos6")) {
                testing_case.set_value(OptValue::from("--unknow-opt"));
            }
            testing_case.add_test(&mut parser)?;
        }

        let args = [
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

        if let Some(ret) = getopt!(args, parser)? {
            for testing_case in testing_cases.iter_mut() {
                testing_case.check_ret(ret.get_set_mut())?;
            }
        }

        assert_eq!(
            parser.get_service().get_noa(),
            &[
                gstr("p"),
                gstr("pos2"),
                gstr("pos3"),
                gstr("pos4"),
                gstr("pos5"),
                gstr("--unknow-opt")
            ]
        );
        Ok(())
    }
}
