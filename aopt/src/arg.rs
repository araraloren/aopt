use std::fmt::Debug;
use std::iter::Iterator;
use std::ops::Deref;
use std::ops::DerefMut;

use regex::Regex;
use ustr::Ustr;

use crate::err::Error;
use crate::err::Result;
use crate::gstr;
use crate::parser::TinyParser;

/// Keeper the information of input command line arguments.
#[derive(Debug, Clone, Default)]
pub struct DataKeeper {
    pub name: Option<Ustr>,

    pub value: Option<Ustr>,

    pub prefix: Option<Ustr>,

    pub disable: bool,
}

/// Parse the input command line item with given regexs, return an [`DataKeeper`].
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
/// use aopt::arg::ArgumentParser;
/// use aopt::parser::TinyParser;
/// use aopt::gstr;
/// use aopt::err::Result;
///
/// fn main() -> Result<()> {
///     let parser = ArgumentParser::new(vec![gstr("--"), gstr("-")])?;
///
///     {// parse option with value
///         let dk = parser.parse(&gstr("--foo=32"))?;
///
///         assert_eq!(dk.prefix, Some(gstr("--")));
///         assert_eq!(dk.name, Some(gstr("foo")));
///         assert_eq!(dk.value, Some(gstr("32")));
///         assert_eq!(dk.disable, false);
///     }
///     {// parse boolean option
///         let dk = parser.parse(&gstr("--/bar"))?;
///
///         assert_eq!(dk.prefix, Some(gstr("--")));
///         assert_eq!(dk.name, Some(gstr("bar")));
///         assert_eq!(dk.value, None);
///         assert_eq!(dk.disable, true);
///     }
///     {// parse other string
///         let dk = parser.parse(&gstr("-=bar"));
///
///         assert!(dk.is_err());
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ArgumentParser {
    regex: Regex,
    prefixs: Vec<Ustr>,
}

impl ArgumentParser {
    pub fn new(prefixs: Vec<Ustr>) -> Result<Self> {
        Ok(Self {
            regex: Regex::new("^(/)?([^=]+)(=(.+))?$").map_err(|e| {
                Error::raise_error(format!("Can not initialize the argument regex!?: {:?}", e))
            })?,
            prefixs,
        })
    }

    pub fn with_regex(mut self, regex: Regex) -> Self {
        self.regex = regex;
        self
    }

    pub fn with_prefixs(mut self, prefixs: Vec<Ustr>) -> Self {
        self.prefixs = prefixs;
        self
    }

    pub fn set_regex(&mut self, regex: Regex) {
        self.regex = regex;
    }

    pub fn set_prefixs(&mut self, prefixs: Vec<Ustr>) {
        self.prefixs = prefixs;
    }

    pub fn get_regex(&self) -> &Regex {
        &self.regex
    }

    pub fn get_prefixs(&self) -> &[Ustr] {
        &self.prefixs
    }
}

impl TinyParser for ArgumentParser {
    type Output = DataKeeper;

    fn parse(&self, pattern: &Ustr) -> Result<Self::Output> {
        for prefix in self.get_prefixs() {
            if pattern.starts_with(prefix.as_str()) {
                let (_, left_part) = pattern.split_at(prefix.len());

                if let Some(cap) = self.get_regex().captures(left_part) {
                    return Ok(Self::Output {
                        name: Some(
                            // result must have name
                            cap.get(2)
                                .map(|v| gstr(v.as_str()))
                                .ok_or_else(|| Error::arg_missing_name(pattern))?,
                        ),
                        prefix: Some(*prefix),
                        value: cap.get(4).map(|v| gstr(v.as_str())),
                        disable: cap.get(1).is_some(),
                    });
                }
            }
        }
        Err(Error::arg_parsing_failed(pattern))
    }
}

/// Argument hold current and next item of command line arguments.
///
/// When parsing the command line option need an argument.
/// The argument of option may embedded in itself.
/// Or we need consume next item as argument of the option.
#[derive(Debug, Clone, Default)]
pub struct Argument {
    pub current: Option<Ustr>,

    pub next: Option<Ustr>,

    data_keeper: DataKeeper,
}

impl Argument {
    pub fn new(current: Option<Ustr>, next: Option<Ustr>) -> Self {
        Self {
            current,
            next,
            ..Self::default()
        }
    }

    pub fn get_data_keeper(&self) -> &DataKeeper {
        &self.data_keeper
    }

    pub fn get_prefix(&self) -> &Option<Ustr> {
        &self.data_keeper.prefix
    }

    pub fn get_name(&self) -> &Option<Ustr> {
        &self.data_keeper.name
    }

    pub fn get_value(&self) -> &Option<Ustr> {
        &self.data_keeper.value
    }

    /// Return true if the option contain deactivate style symbol '/'
    pub fn is_disabled(&self) -> bool {
        self.data_keeper.disable
    }

    /// Call [`parse_argument`] parse the command line item with given regexs.
    ///
    /// # Returns
    ///
    /// Will save the [`DataKeeper`] to self and return `Ok(true)` when successed.
    /// Return `Ok(false)` when current item is [`None`].
    ///
    /// # Errors
    ///
    /// - [`ArgumentError::MissingPrefix`](crate::err::ArgumentError::MissingPrefix)
    ///
    /// When the result not have a valid prefix.
    ///
    /// - [`ArgumentError::MissingName`](crate::err::ArgumentError::MissingName)
    ///
    /// When the result not have a valid name.
    pub fn parse<TP: TinyParser<Output = DataKeeper>>(&mut self, tiny_parser: &TP) -> Result<bool> {
        if let Some(pattern) = &self.current {
            self.data_keeper = tiny_parser.parse(pattern)?;

            return Ok(true);
        }
        Ok(false)
    }
}

/// The wrapper of command line items, it will output [`Argument`].
///
/// # Example
/// ```rust
/// use aopt::arg::ArgStream;
/// use ustr::Ustr;
/// use aopt::gstr;
/// use aopt::err::Result;
///
/// fn main() -> Result<()> {
///     let args = ["-a", "v1", "--aopt", "p1", "p2", "--bopt", "v2"]
///         .iter()
///         .map(|&v| String::from(v));
///     let mut stream = ArgStream::from(args);
///     let next = stream.next().unwrap();
///
///     assert_eq!(next.current, Some(gstr("-a")));
///     assert_eq!(next.next, Some(gstr("v1")));
///     stream.next();
///     let next = stream.next().unwrap();
///
///     assert_eq!(next.current, Some(gstr("--aopt")));
///     assert_eq!(next.next, Some(gstr("p1")));
///     let next = stream.next().unwrap();
///
///     assert_eq!(next.current, Some(gstr("p1")));
///     assert_eq!(next.next, Some(gstr("p2")));
///     stream.next();
///     let next = stream.next().unwrap();
///
///     assert_eq!(next.current, Some(gstr("--bopt")));
///     assert_eq!(next.next, Some(gstr("v2")));
///     let next = stream.next().unwrap();
///
///     assert_eq!(next.current, Some(gstr("v2")));
///     assert_eq!(next.next, None);  
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ArgStream {
    args: Vec<Ustr>,
    curr: usize,
}

impl ArgStream {
    pub fn new<I, ITER>(iter: ITER) -> Self
    where
        I: Into<String>,
        ITER: Iterator<Item = I>,
    {
        let iter = iter.map(|v| gstr(&v.into()));
        Self {
            args: iter.collect(),
            curr: 0,
        }
    }

    pub fn current(&self) -> usize {
        self.curr
    }
}

impl Deref for ArgStream {
    type Target = [Ustr];

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl DerefMut for ArgStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.args
    }
}

impl Default for ArgStream {
    fn default() -> Self {
        ArgStream::new(std::env::args())
    }
}

impl Iterator for ArgStream {
    type Item = Argument;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;
        if curr < self.len() {
            self.curr += 1;
            Some(Argument::new(
                self.args.get(curr).copied(),
                self.args.get(curr + 1).copied(),
            ))
        } else {
            None
        }
    }
}

impl ExactSizeIterator for ArgStream {
    fn len(&self) -> usize {
        self.args.len()
    }
}

impl<T: Iterator<Item = String>> From<T> for ArgStream {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

#[cfg(test)]
mod test {

    use super::ArgStream;
    use super::ArgumentParser;
    use crate::gstr;
    use crate::parser::TinyParser;
    use ustr::Ustr;

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
            .iter()
            .map(|&v| String::from(v));
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
                ArgStream::new(data),
                vec![gstr("--"), gstr("-")],
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
            .iter()
            .map(|&v| String::from(v));
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
                ArgStream::new(data),
                vec![gstr("+"), gstr("")],
                &data_check,
                &check,
            );
        }
    }

    fn testing_one_iterator(
        argstream: ArgStream,
        prefixs: Vec<Ustr>,
        data_check: &Vec<String>,
        check: &Vec<Vec<&str>>,
    ) {
        let default_str = gstr("");
        let default_data = String::from("");
        let default_item = "";
        let parser = ArgumentParser::new(prefixs).unwrap();

        for ((index, mut arg), check_item) in argstream.enumerate().zip(check.iter()) {
            assert_eq!(
                arg.current.as_ref().unwrap_or(&default_str),
                data_check.get(index).unwrap_or(&default_data)
            );
            assert_eq!(
                arg.next.as_ref().unwrap_or(&default_str),
                data_check.get(index + 1).unwrap_or(&default_data)
            );
            if let Ok(ret) = arg.parse(&parser) {
                if ret {
                    assert_eq!(
                        arg.get_prefix().as_ref().unwrap_or(&default_str),
                        check_item.get(0).unwrap_or(&default_item)
                    );
                    assert_eq!(
                        arg.get_name().as_ref().unwrap_or(&default_str),
                        check_item.get(1).unwrap_or(&default_item)
                    );
                    assert_eq!(
                        arg.get_value().as_ref().unwrap_or(&default_str),
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

            let parser = ArgumentParser::new(vec![gstr("--"), gstr("-"), gstr("")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &parser, case.1);
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

            let parser = ArgumentParser::new(vec![gstr("--"), gstr("-")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &parser, case.1);
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

            let parser = ArgumentParser::new(vec![gstr("++"), gstr("+")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &parser, case.1);
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

            let parser = ArgumentParser::new(vec![gstr("++"), gstr("+")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &parser, case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Ustr,
        parser: &ArgumentParser,
        except: Option<(Option<&str>, Option<&str>, Option<&str>, bool)>,
    ) {
        let ret = parser.parse(&pattern);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            if except.is_none() {
                panic!("----> {:?} {:?}", except, &dk);
            }

            let default = gstr("");

            if let Some(except) = except {
                assert_eq!(
                    except.0.unwrap_or(""),
                    dk.prefix.unwrap_or(default).as_ref()
                );
                assert_eq!(except.1.unwrap_or(""), dk.name.unwrap_or(default).as_ref());
                assert_eq!(except.2.unwrap_or(""), dk.value.unwrap_or(default).as_ref());
                assert_eq!(except.3, dk.disable);
            }
        } else {
            if except.is_some() {
                eprintln!("----> {:?}", except);
            }
            assert!(except.is_none());
        }
    }
}
