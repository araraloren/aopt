mod argument;
mod parser;

use std::fmt::Debug;
use std::iter::Iterator;
use std::ops::Deref;
use std::ops::DerefMut;

use ustr::Ustr;

use crate::gstr;

pub use argument::Argument;
pub use parser::parse_argument;
pub use parser::DataKeeper;

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
    use crate::gstr;
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
                &vec![gstr("--"), gstr("-")],
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
                &vec![gstr("+"), gstr("")],
                &data_check,
                &check,
            );
        }
    }

    fn testing_one_iterator<'pre, 'vec: 'pre>(
        argstream: ArgStream,
        prefixs: &'vec Vec<Ustr>,
        data_check: &Vec<String>,
        check: &Vec<Vec<&str>>,
    ) {
        let default_str = gstr("");
        let default_data = String::from("");
        let default_item = "";

        for ((index, mut arg), check_item) in argstream.enumerate().zip(check.iter()) {
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
