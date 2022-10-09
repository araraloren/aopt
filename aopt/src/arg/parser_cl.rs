use regex::Regex;

use crate::astr;
use crate::Error;
use crate::Str;

use super::ArgParser;

#[derive(Debug, Clone, Default)]
pub struct CLOpt {
    pub name: Option<Str>,

    pub value: Option<Str>,

    pub prefix: Option<Str>,

    pub disable: bool,
}

impl CLOpt {
    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn val(&self) -> Option<&Str> {
        self.value.as_ref()
    }

    pub fn pre(&self) -> Option<&Str> {
        self.prefix.as_ref()
    }

    pub fn dsb(&self) -> bool {
        self.disable
    }
}

/// Parse the input command line item with given regexs, return an [`CLOpt`].
///
/// The struct of the input option string are:
///
/// ```!
/// [--][/][option][=][value]
///   |  |     |    |    |
///   |  |     |    |    |
///   |  |     |    |    The value part, it is optional.
///   |  |     |    |
///   |  |     |    The delimiter of option name and value.
///   |  |     |    
///   |  |     The option name part, it must be provide by user.
///   |  |
///   |  The disable symbol, generally it is using for boolean option.
///   |  
///   The prefix of option.
/// ```
///
/// # Example
///
/// ```rust
/// extern crate aopt as test_crate;
///
/// use test_crate::arg::CLOptParser;
/// use test_crate::arg::ArgParser;
/// use test_crate::astr;
/// use test_crate::err::Result;
///
/// fn main() -> Result<()> {
///     let mut parser = CLOptParser::default();
///     let prefixs = vec![astr("--"), astr("-")];
///
///     {// parse option with value
///         let output = parser.parse(astr("--foo=32"), &prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("foo")));
///         assert_eq!(output.value, Some(astr("32")));
///         assert_eq!(output.disable, false);
///     }
///     {// parse boolean option
///         let output = parser.parse(astr("--/bar"), &prefixs)?;
///
///         assert_eq!(output.prefix, Some(astr("--")));
///         assert_eq!(output.name, Some(astr("bar")));
///         assert_eq!(output.value, None);
///         assert_eq!(output.disable, true);
///     }
///     {// parse other string
///         let output = parser.parse(astr("-=bar"), &prefixs);
///
///         assert!(output.is_err());
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct CLOptParser(Regex);

impl CLOptParser {
    pub fn new(regex: Regex) -> Self {
        Self(regex)
    }

    pub fn regex(&self) -> &Regex {
        &self.0
    }
}

impl Default for CLOptParser {
    fn default() -> Self {
        // the regex is always valid, so just unwrap
        Self::new(Regex::new(r"^(/)?([^=]+)(=(.+))?$").unwrap())
    }
}

impl ArgParser for CLOptParser {
    type Output = CLOpt;

    type Error = Error;

    fn parse(&mut self, pattern: Str, prefixs: &[Str]) -> Result<Self::Output, Self::Error> {
        for prefix in prefixs {
            if pattern.starts_with(prefix.as_str()) {
                let (_, left_part) = pattern.split_at(prefix.len());

                if let Some(cap) = self.regex().captures(left_part) {
                    //log!(format!("Regex result -> {:?}", cap));
                    return Ok(Self::Output {
                        name: Some(
                            // result must have name
                            cap.get(2)
                                .map(|v| astr(v.as_str()))
                                .ok_or_else(|| Error::arg_missing_name(pattern.clone()))?,
                        ),
                        prefix: Some(prefix.clone()),
                        value: cap.get(4).map(|v| astr(v.as_str())),
                        disable: cap.get(1).is_some(),
                    });
                }
            }
        }
        Err(Error::arg_parsing_failed(pattern))
    }
}

#[cfg(test)]
mod test {

    use super::ArgParser;
    use super::CLOptParser;
    use crate::arg::args::ArgsIter;
    use crate::arg::Args;
    use crate::astr;
    use crate::Str;

    #[test]
    fn make_sure_arg_stream_work() {
        {
            // test1
            let data = [
                "cpp",
                "-d",
                "-i=iostream",
                "-L",
                "ncurses",
                "--output",
                "download.cpp",
                "--compile",
                "--wget",
                "https://example.com/template.cpp",
            ]
            .into_iter();

            let data_check = data.clone().collect();
            let check = vec![
                vec![],
                vec!["-", "d"],
                vec!["-", "i", "iostream"],
                vec!["-", "L"],
                vec![],
                vec!["--", "output"],
                vec![],
                vec!["--", "compile"],
                vec!["--", "wget"],
                vec![],
            ];

            testing_one_iterator(
                Args::new(data).iter(),
                vec![astr("--"), astr("-")],
                &data_check,
                &check,
            );
        }
        {
            // test2
            let data = [
                "c",
                "+d",
                "std=c11",
                "i=stdlib.h",
                "L",
                "ncurses",
                "output",
                "download.c",
                "+compile",
                "+wget",
                "https://example.com/template.c",
            ]
            .into_iter();
            let data_check = data.clone().collect();
            let check = vec![
                vec!["", "c"],
                vec!["+", "d"],
                vec!["", "std", "c11"],
                vec!["", "i", "stdlib.h"],
                vec!["", "L"],
                vec!["", "ncurses"],
                vec!["", "output"],
                vec!["", "download.c"],
                vec!["+", "compile"],
                vec!["+", "wget"],
                vec!["", "https://example.com/template.c"],
            ];

            testing_one_iterator(
                Args::new(data).iter(),
                vec![astr("+"), astr("")],
                &data_check,
                &check,
            );
        }
    }

    fn testing_one_iterator<T: Into<Str>, I: Iterator<Item = T> + Clone>(
        mut args: ArgsIter<I>,
        prefixs: Vec<Str>,
        data_check: &Vec<&str>,
        check: &Vec<Vec<&str>>,
    ) {
        let default_str = astr("");
        let default_data = "";
        let default_item = "";
        let mut parser = CLOptParser::default();

        while let Some(_) = args.next() {
            let index = args.idx();

            assert_eq!(
                args.cur().unwrap_or(&default_str),
                data_check.get(index).unwrap_or(&default_data)
            );
            assert_eq!(
                args.arg().unwrap_or(&default_str),
                data_check.get(index + 1).unwrap_or(&default_data)
            );
            if let Some(curr) = args.cur() {
                if let Ok(ret) = parser.parse(curr.clone(), &prefixs) {
                    let check_item = &check[index];

                    assert_eq!(
                        ret.prefix.as_ref().unwrap_or(&default_str),
                        check_item.get(0).unwrap_or(&default_item)
                    );
                    assert_eq!(
                        ret.name.as_ref().unwrap_or(&default_str),
                        check_item.get(1).unwrap_or(&default_item)
                    );
                    assert_eq!(
                        ret.value.as_ref().unwrap_or(&default_str),
                        check_item.get(2).unwrap_or(&default_item)
                    );
                }
            }
        }
    }

    #[test]
    fn test_for_input_parser() {
        {
            // test 1
            let test_cases = vec![
                ("", None),
                ("-a", Some((Some("-"), Some("a"), None, false))),
                ("-/a", Some((Some("-"), Some("a"), None, true))),
                ("-a=b", Some((Some("-"), Some("a"), Some("b"), false))),
                ("--foo", Some((Some("--"), Some("foo"), None, false))),
                ("--/foo", Some((Some("--"), Some("foo"), None, true))),
                (
                    "--foo=bar",
                    Some((Some("--"), Some("foo"), Some("bar"), false)),
                ),
                ("a", Some((Some(""), Some("a"), None, false))),
                ("/a", Some((Some(""), Some("a"), None, true))),
                ("a=b", Some((Some(""), Some("a"), Some("b"), false))),
                ("foo", Some((Some(""), Some("foo"), None, false))),
                ("/foo", Some((Some(""), Some("foo"), None, true))),
                ("foo=bar", Some((Some(""), Some("foo"), Some("bar"), false))),
                ("--=xar", Some((Some("-"), Some("-"), Some("xar"), false))),
                ("-foo=", None),
            ];

            let mut parser = CLOptParser::default();

            for case in test_cases.iter() {
                try_to_verify_one_task(
                    astr(case.0),
                    &mut parser,
                    vec![astr("--"), astr("-"), astr("")],
                    case.1,
                );
            }
        }
        {
            // test 2
            let test_cases = vec![
                ("", None),
                ("-a", Some((Some("-"), Some("a"), None, false))),
                ("-/a", Some((Some("-"), Some("a"), None, true))),
                ("-a=b", Some((Some("-"), Some("a"), Some("b"), false))),
                ("--foo", Some((Some("--"), Some("foo"), None, false))),
                ("--/foo", Some((Some("--"), Some("foo"), None, true))),
                (
                    "--foo=bar",
                    Some((Some("--"), Some("foo"), Some("bar"), false)),
                ),
                ("a", None),
                ("/a", None),
                ("a=b", None),
                ("foo", None),
                ("/foo", None),
                ("foo=bar", None),
                ("--=xar", Some((Some("-"), Some("-"), Some("xar"), false))),
                ("-foo=", None),
            ];

            let mut parser = CLOptParser::default();

            for case in test_cases.iter() {
                try_to_verify_one_task(
                    astr(case.0),
                    &mut parser,
                    vec![astr("--"), astr("-")],
                    case.1,
                );
            }
        }
        {
            // test 3
            let test_cases = vec![
                ("", None),
                ("+a", Some((Some("+"), Some("a"), None, false))),
                ("+/a", Some((Some("+"), Some("a"), None, true))),
                ("+a=b", Some((Some("+"), Some("a"), Some("b"), false))),
                ("++foo", Some((Some("++"), Some("foo"), None, false))),
                ("++/foo", Some((Some("++"), Some("foo"), None, true))),
                (
                    "++foo=bar",
                    Some((Some("++"), Some("foo"), Some("bar"), false)),
                ),
                ("a", None),
                ("/a", None),
                ("a=b", None),
                ("foo", None),
                ("/foo", None),
                ("foo=bar", None),
                ("++=xar", Some((Some("+"), Some("+"), Some("xar"), false))),
                ("+foo=", None),
            ];

            let mut parser = CLOptParser::default();

            for case in test_cases.iter() {
                try_to_verify_one_task(
                    astr(case.0),
                    &mut parser,
                    vec![astr("++"), astr("+")],
                    case.1,
                );
            }
        }
        {
            // test 3
            let test_cases = vec![
                ("", None),
                ("+选项", Some((Some("+"), Some("选项"), None, false))),
                ("+/选项", Some((Some("+"), Some("选项"), None, true))),
                (
                    "+选项=值",
                    Some((Some("+"), Some("选项"), Some("值"), false)),
                ),
                (
                    "++选项foo",
                    Some((Some("++"), Some("选项foo"), None, false)),
                ),
                (
                    "++/选项foo",
                    Some((Some("++"), Some("选项foo"), None, true)),
                ),
                (
                    "++选项=bar",
                    Some((Some("++"), Some("选项"), Some("bar"), false)),
                ),
                ("选项", None),
                ("/选项", None),
                ("选项=b", None),
                ("选项", None),
                ("/选项", None),
                ("选项=bar", None),
                ("++=xar", Some((Some("+"), Some("+"), Some("xar"), false))),
                ("+选项=", None),
            ];

            let mut parser = CLOptParser::default();

            for case in test_cases.iter() {
                try_to_verify_one_task(
                    astr(case.0),
                    &mut parser,
                    vec![astr("++"), astr("+")],
                    case.1,
                );
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Str,
        parser: &mut CLOptParser,
        prefixs: Vec<Str>,
        except: Option<(Option<&str>, Option<&str>, Option<&str>, bool)>,
    ) {
        let ret = parser.parse(pattern, &prefixs);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            let default = astr("");

            if let Some(except) = except {
                assert_eq!(
                    except.0.unwrap_or(""),
                    dk.prefix.unwrap_or(default.clone()).as_ref()
                );
                assert_eq!(
                    except.1.unwrap_or(""),
                    dk.name.unwrap_or(default.clone()).as_ref()
                );
                assert_eq!(except.2.unwrap_or(""), dk.value.unwrap_or(default).as_ref());
                assert_eq!(except.3, dk.disable);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
