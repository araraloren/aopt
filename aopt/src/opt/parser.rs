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
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
///     let parser = StrParser::default();
///     let ret = parser.parse_opt("--aopt=t!".into())?;
///
///     assert_eq!(ret.name() , Some("--aopt"));
///     assert_eq!(ret.ctor(), Some("t"));
///     assert_eq!(ret.force(), Some(true));
///     assert_eq!(ret.index(), None);
///
///     let ret = parser.parse_opt("bopt=t@[1,2,3]".into())?;
///
///     assert_eq!(ret.name(), Some("bopt"));
///     assert_eq!(ret.ctor(), Some("t"));
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

impl StrParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_creator_string(&self, dat: &str) -> Result<ConstrctInfo, Error> {
        use neure::prelude::*;

        let name = ['=', '!', '*', '@', ';', ':'].not().many1();
        let aliases = name.sep(";");
        let parser = name.opt().if_then(";", aliases);

        let ctor = neu::alphabetic().many1();
        let parser = parser.if_then("=", ctor);

        let opt = "!".or("*").opt();
        let parser = parser.then(opt);

        let index = '@'.or(':').not().many1();
        let parser = parser.if_then("@", index);

        let help = regex::consume_all();
        let parser = parser.if_then(":", help);

        let parser = parser.suffix(regex::end()).prefix(regex::start());

        let to_string = |v: &str| v.trim().to_string();

        let (((((name, aliases), ctor), opt), index), help) = CharsCtx::new(dat)
            .ctor(&parser)
            .map_err(|_| Error::create_str(dat, "can not parsing string"))?;

        let mut ci = ConstrctInfo::default();

        ci = ci.with_name(name.map(to_string));
        ci = ci.with_alias(aliases.map(|v| v.iter().copied().map(to_string).collect()));
        ci = ci.with_ctor(ctor.map(to_string));
        ci = ci.with_force(opt.map(|v| v == "!"));
        ci = ci.with_index(if let Some(index) = index {
            Some(Index::parse(index)?)
        } else {
            None
        });
        ci = ci.with_help(help.map(to_string));
        Ok(ci)
    }
}

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
