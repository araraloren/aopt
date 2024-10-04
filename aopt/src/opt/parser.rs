use std::cell::RefCell;

use neure::neure;
use neure::regex;
use neure::CharsCtx;
use neure::MatchPolicy;
use neure::SpanStore;
use neure::SpanStorer;

use super::{ConstrctInfo, OptParser};
use crate::opt::Index;
use crate::Error;

/// Parse the option string with given prefixes, return an [`ConstrctInfo`].
///
/// The struct of the option string are:
///
/// ```plaintext
/// [--option][=][type][!*][@index]
///      |     |    |   |   |
///      |     |    |   |   |
///      |     |    |   |   |
///      |     |    |   |   The index part of option. Here are all the possible string:
///      |     |    |   |   @0 means first position
///      |     |    |   |   @-0 means last position
///      |     |    |   |   @[1, 2, 3] means the position 1, 2 and 3
///      |     |    |   |   @-[1, 2] means except the position 1, 2
///      |     |    |   |   @>2 means position that bigger than 2
///      |     |    |   |   @<3 means position less than 3
///      |     |    |   |   @* means all the position
///      |     |    |   |
///      |     |    |   Indicate the option wether is force required(!) or not(*).
///      |     |    |
///      |     |    |
///      |     |    |
///      |     |    The type name of option.
///      |     |    
///      |     The delimiter of option name and type.
///      |
///      The option name part, it must be provide by user.
/// ```
///
/// # Example
///
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::astr;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
///     let parser = StrParser::default();
///     let ret = parser.parse_opt("--aopt=t!".into())?;
///
///     assert_eq!(ret.name() , Some(&a"--aopt")));
///     assert_eq!(ret.ctor(), Some(&a"t")));
///     assert_eq!(ret.force(), Some(true));
///     assert_eq!(ret.index(), None);
///
///     let ret = parser.parse_opt("bopt=t@[1,2,3]".into())?;
///
///     assert_eq!(ret.name(), Some(&a"bopt")));
///     assert_eq!(ret.ctor(), Some(&a"t")));
///     assert_eq!(ret.force(), None);
///     assert_eq!(ret.index(), Some(&Index::list(vec![1, 2, 3])));
///
/// #   Ok(())
/// # }
/// ```
///
/// For more examples, please reference test case [`test_option_str_parser`](../../src/aopt/set/parser.rs.html#542).
///
#[derive(Debug, Default)]
pub struct StrParser;

thread_local! {
    static STR_PARSER: RefCell<SpanStorer> = RefCell::new(SpanStorer::new(KEY_TOTAL));
}

impl StrParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_ctx(storer: &mut SpanStorer, str: &str) -> Result<(), neure::err::Error> {
        let start = neure::start();
        let end = neure::end();
        let name = neure!([^'=' '!' '*' '@' ';' ':']+);
        let semi = neure!(';');
        let equal = neure!('=');
        let ty = neure!([a-z A-Z]+);
        let optional = neure!(['!' '*']);
        let at = neure!('@');
        let index = neure!([^ '@' ':']+);
        let colon = neure!(':');
        let usage = neure!(.+);
        let space = neure!(*);
        let parser = |storer: &mut SpanStorer, str| -> Result<(), neure::err::Error> {
            let mut ctx = CharsCtx::new(str);

            ctx.try_mat(&start)?;
            if ctx.cap(KEY_NAME, storer, &name) {
                // name
                while ctx.mat(&semi) {
                    ctx.cap(KEY_ALIAS, storer, &name);
                }
            }
            if ctx.mat(&equal) {
                // = type
                ctx.try_cap(KEY_CTOR, storer, &ty)?;
            }
            ctx.cap(KEY_OPTIONAL, storer, &optional); // ! or *
            if ctx.mat(&at) {
                // @index
                ctx.try_cap(KEY_INDEX, storer, &index)?;
            }
            if ctx.mat(&colon) {
                ctx.mat(&space);
                ctx.try_cap(KEY_HELP, storer, &usage)?;
            }
            ctx.try_mat(&end)?;
            Ok(())
        };

        parser(storer, str)
    }

    pub fn parse_creator_string(&self, pattern: &str) -> Result<ConstrctInfo, Error> {
        STR_PARSER
            .try_with(|storer| {
                if Self::parse_ctx(storer.borrow_mut().reset(), pattern).is_ok() {
                    let mut force = None;
                    let mut idx = None;
                    let mut alias = None;
                    let storer = storer.borrow();
                    let name = storer.substr(pattern, KEY_NAME, 0).ok();
                    let help = storer.substr(pattern, KEY_HELP, 0).ok();
                    let ctor = storer.substr(pattern, KEY_CTOR, 0).ok();

                    if let Ok(opt) = storer.substr(pattern, KEY_OPTIONAL, 0) {
                        match opt {
                            "!" => {
                                force = Some(true);
                            }
                            "*" => {
                                force = Some(false);
                            }
                            _ => {
                                unreachable!(
                                    "Oops ?!! Regex make sure option string correctly: {}",
                                    &pattern
                                )
                            }
                        }
                    }
                    if let Ok(vals) = storer.substrs(pattern, KEY_ALIAS) {
                        alias = Some(
                            vals.filter(|v| !v.trim().is_empty())
                                .map(|v| String::from(v.trim()))
                                .collect(),
                        );
                    }
                    if let Ok(index) = storer.substr(pattern, KEY_INDEX, 0) {
                        idx = Some(Index::parse(index)?);
                    }
                    Ok(ConstrctInfo::default()
                        .with_force(force)
                        .with_index(idx)
                        .with_name(name.map(|v| String::from(v.trim())))
                        .with_help(help.map(|v| String::from(v.trim())))
                        .with_ctor(ctor.map(|v| String::from(v.trim())))
                        .with_alias(alias))
                } else {
                    Err(Error::create_str(pattern, "can not parsing string"))
                }
            })
            .map_err(|e| Error::thread_local_access().cause_by(e.into()))?
    }
}

const KEY_NAME: usize = 0;
const KEY_ALIAS: usize = 1;
const KEY_CTOR: usize = 2;
const KEY_OPTIONAL: usize = 3;
const KEY_INDEX: usize = 4;
const KEY_HELP: usize = 5;
const KEY_TOTAL: usize = KEY_HELP + 1;

impl OptParser for StrParser {
    type Output = ConstrctInfo;

    type Error = Error;

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error> {
        if pattern.trim().is_empty() {
            Ok(Self::Output::default())
        } else {
            self.parse_creator_string(pattern)
        }
    }
}

#[cfg(test)]
mod test {

    use crate::prelude::*;

    #[test]
    fn test_str_parser() {
        let options = [
            "-b",
            "--bool",
            "bool",
            "-b;--bool",
            "-?;-h;--help",
            "--bool;-b",
            "b;bool",
            "-b;bool",
            "-/b;--/bool",
            "-/b;bool",
            "-b=i",
            "--bool=u",
            "bool=s",
            "-b;--bool=b",
            "-?;-h;--help=p",
            "--bool;-b=c",
            "b;bool=m",
            "-b;bool=f",
            "-/b;--/bool=i",
            "-/b;bool=a",
            "",
        ];
        let options_test = [
            (Some("-b"), None, None),
            (Some("--bool"), None, None),
            (Some("bool"), None, None),
            (Some("-b"), Some(vec!["--bool"]), None),
            (Some("-?"), Some(vec!["-h", "--help"]), None),
            (Some("--bool"), Some(vec!["-b"]), None),
            (Some("b"), Some(vec!["bool"]), None),
            (Some("-b"), Some(vec!["bool"]), None),
            (Some("-/b"), Some(vec!["--/bool"]), None),
            (Some("-/b"), Some(vec!["bool"]), None),
            (Some("-b"), None, Some("i")),
            (Some("--bool"), None, Some("u")),
            (Some("bool"), None, Some("s")),
            (Some("-b"), Some(vec!["--bool"]), Some("b")),
            (Some("-?"), Some(vec!["-h", "--help"]), Some("p")),
            (Some("--bool"), Some(vec!["-b"]), Some("c")),
            (Some("b"), Some(vec!["bool"]), Some("m")),
            (Some("-b"), Some(vec!["bool"]), Some("f")),
            (Some("-/b"), Some(vec!["--/bool"]), Some("i")),
            (Some("-/b"), Some(vec!["bool"]), Some("a")),
            (None, None, None),
        ];
        let helps = [": This is an option help message", ""];
        let helps_test = [Some("This is an option help message"), None];
        let forces = ["!", "*", ""];
        let forces_test = [Some(true), Some(false), None];
        let positions = [
            "@1",
            "@68",
            "@-6",
            "@+42",
            "@1..5",
            "@..8",
            "@2..",
            "@[1,3,5]",
            "@+[2,3,4]",
            "@-[3,56]",
            "@*",
            "",
        ];
        let positions_test = [
            Some(Index::forward(1)),
            Some(Index::forward(68)),
            Some(Index::backward(6)),
            Some(Index::forward(42)),
            Some(Index::range(Some(1), Some(5))),
            Some(Index::range(None, Some(8))),
            Some(Index::range(Some(2), None)),
            Some(Index::list(vec![1, 3, 5])),
            Some(Index::list(vec![2, 3, 4])),
            Some(Index::except(vec![3, 56])),
            Some(Index::anywhere()),
            None,
        ];
        let parser = StrParser;

        for (option, option_test) in options.iter().zip(options_test.iter()) {
            for (help, help_test) in helps.iter().zip(helps_test.iter()) {
                for (force, force_test) in forces.iter().zip(forces_test.iter()) {
                    for (position, position_test) in positions.iter().zip(positions_test.iter()) {
                        let creator = format!("{}{}{}{}", option, force, position, help);

                        println!("\"{}\",", creator);
                        if let Ok(cap) = parser.parse_opt(&creator) {
                            assert_eq!(option_test.0, cap.name());
                            assert_eq!(
                                option_test.1,
                                cap.alias().map(|v| v.iter().map(|v| v.as_ref()).collect())
                            );
                            assert_eq!(help_test, &cap.help());
                            assert_eq!(force_test, &cap.force());
                            assert_eq!(position_test.as_ref(), cap.index());
                            assert_eq!(option_test.2, cap.ctor());
                        } else {
                            assert!(
                                option_test.0.is_none(),
                                "{}{}{}{}",
                                option,
                                force,
                                position,
                                help
                            );
                            assert!(option_test.1.is_none());
                        }
                    }
                }
            }
        }
    }
}
