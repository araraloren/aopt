pub mod argument;
pub mod parser;

use std::convert::From;
use std::fmt::Debug;
use std::iter::Iterator;
use std::slice::{Iter, IterMut};

use crate::OptStr;

pub use argument::Argument;

#[derive(Debug, Default)]
pub struct ArgStream {
    args: Vec<Argument>,
    index: usize,
}

impl ArgStream {
    pub fn new(args: impl Iterator<Item = String>) -> Self {
        Self {
            args: Self::iterator_to_args(args),
            index: 0,
        }
    }

    pub fn set_args(&mut self, args: impl Iterator<Item = String>) -> &mut Self {
        self.args = Self::iterator_to_args(args);
        self
    }

    pub fn iter(&self) -> Iter<'_, Argument> {
        self.args.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Argument> {
        self.args.iter_mut()
    }

    fn iterator_to_args<Iter>(mut iter: Iter) -> Vec<Argument>
    where
        Iter: Iterator<Item = String>,
    {
        let mut ret = vec![];
        let mut current = iter.next();

        while current.is_some() {
            let next = iter.next();

            ret.push(Argument::new(
                Self::map_one_item(current),
                Self::map_one_item(next.clone()),
            ));
            current = next;
        }
        ret
    }

    fn map_one_item(item: Option<String>) -> Option<OptStr> {
        item.map_or(None, |v| Some(OptStr::from(&v)))
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }
}

impl<'str, 'nv, 'pre, It: Iterator<Item = String>> From<It> for ArgStream {
    fn from(iter: It) -> Self {
        Self {
            args: Self::iterator_to_args(iter),
            index: 0,
        }
    }
}

#[cfg(test)]
mod test {

    use super::ArgStream;
    use crate::OptStr;

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
                &vec![OptStr::from("--"), OptStr::from("-")],
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
                &vec![OptStr::from("+"), OptStr::from("")],
                &data_check,
                &check,
            );
        }
    }

    fn testing_one_iterator<'pre, 'vec: 'pre>(
        mut argstream: ArgStream,
        prefixs: &'vec Vec<OptStr>,
        data_check: &Vec<String>,
        check: &Vec<Vec<&str>>,
    ) {
        let default_str = OptStr::from("");
        let default_data = String::from("");
        let default_item = "";

        for ((index, arg), check_item) in argstream.iter_mut().enumerate().zip(check.iter()) {
            assert_eq!(
                arg.current.as_ref().unwrap_or(&default_str),
                data_check.get(index).unwrap_or(&default_data)
            );
            assert_eq!(
                arg.next.as_ref().unwrap_or(&default_str),
                data_check.get(index + 1).unwrap_or(&default_data)
            );
            if let Ok(ret) = arg.parse(prefixs) {
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
}
