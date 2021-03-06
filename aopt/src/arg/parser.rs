use ustr::Ustr;

use crate::err::Error;
use crate::err::Result;
use crate::pat::ParseIndex;
use crate::pat::ParserPattern;

/// Parse the input command line item with given prefixs, return an [`DataKeeper`].
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
/// use aopt::arg::parse_argument;
/// use ustr::Ustr;
/// use aopt::gstr;
/// use aopt::err::Result;
///
/// fn main() -> Result<()> {
///     let prefix = &[gstr("--"), gstr("-")];
///
///     {// parse option with value
///         let dk = parse_argument(gstr("--foo=32"), prefix)?;
///
///         assert_eq!(dk.prefix, Some(gstr("--")));
///         assert_eq!(dk.name, Some(gstr("foo")));
///         assert_eq!(dk.value, Some(gstr("32")));
///         assert_eq!(dk.disable, false);
///     }
///     {// parse boolean option
///         let dk = parse_argument(gstr("--/bar"), prefix)?;
///
///         assert_eq!(dk.prefix, Some(gstr("--")));
///         assert_eq!(dk.name, Some(gstr("bar")));
///         assert_eq!(dk.value, None);
///         assert_eq!(dk.disable, true);
///     }
///     {// parse other string
///         let dk = parse_argument(gstr("-=bar"), prefix)?;
///
///         assert_eq!(dk.prefix, Some(gstr("-")));
///         assert_eq!(dk.name, None);
///         assert_eq!(dk.value, Some(gstr("bar")));
///         assert_eq!(dk.disable, false);
///     }
///     Ok(())
/// }
/// ```
pub fn parse_argument(pattern: Ustr, prefix: &[Ustr]) -> Result<DataKeeper> {
    let pattern = ParserPattern::new(pattern, prefix);
    let mut index = ParseIndex::new(pattern.len());
    let mut data_keeper = DataKeeper::default();

    let res = State::default().parse(&mut index, &pattern, &mut data_keeper)?;

    if res {
        trace!(
            ?pattern,
            ?prefix,
            ?data_keeper,
            "parsing argument successed"
        );
        return Ok(data_keeper);
    }
    error!(?pattern, ?prefix, ?index, "parsing argument failed");
    Err(Error::arg_parsing_failed(pattern.get_pattern()))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    PreCheck,
    Prefix,
    Disable,
    Name,
    Equal,
    Value,
    End,
}

#[derive(Debug, Clone, Default)]
pub struct DataKeeper {
    pub name: Option<Ustr>,

    pub value: Option<Ustr>,

    pub prefix: Option<Ustr>,

    pub disable: bool,
}

impl Default for State {
    fn default() -> Self {
        Self::PreCheck
    }
}

const DEACTIVATE_STYLE_CHAR: char = '/';
const VALUE_SPLIT_CHAR: char = '=';

impl State {
    pub fn self_transition<'pre>(&mut self, index: &ParseIndex, pattern: &ParserPattern<'pre>) {
        let next_state = {
            match self.clone() {
                Self::PreCheck => Self::Prefix,
                Self::Prefix => {
                    if pattern.starts(DEACTIVATE_STYLE_CHAR, index.get()) {
                        Self::Disable
                    } else {
                        Self::Name
                    }
                }
                Self::Disable => Self::Name,
                Self::Name => {
                    if pattern.starts(VALUE_SPLIT_CHAR, index.get()) {
                        Self::Equal
                    } else {
                        Self::End
                    }
                }
                Self::Equal => Self::Value,
                Self::Value => Self::End,
                Self::End => {
                    unreachable!("The end state can't going on!");
                }
            }
        };
        trace!("transition state from '{:?}' to '{:?}'", self, next_state);
        *self = next_state;
    }

    pub fn parse<'pre>(
        mut self,
        index: &mut ParseIndex,
        pattern: &ParserPattern<'pre>,
        data_keeper: &mut DataKeeper,
    ) -> Result<bool> {
        let current_state = self.clone();

        match current_state {
            Self::PreCheck => {
                if pattern.is_empty() {
                    warn!("got an empty pattern");
                    return Ok(false);
                }
            }
            Self::Prefix => {
                if let Some(prefix) = pattern.get_prefix() {
                    data_keeper.prefix = Some(*prefix);
                    index.inc(prefix.chars().count());
                }
            }
            Self::Disable => {
                data_keeper.disable = true;
                index.inc(1);
            }
            Self::Name => {
                let start = index.get();

                // get the chars until we meet '=' or reach the end
                for (cur, ch) in pattern.get_chars(start).iter().enumerate() {
                    let mut name_end = 0;
                    // the name not include '=', so > 1
                    if *ch == VALUE_SPLIT_CHAR {
                        if cur >= 1 {
                            name_end = start + cur;
                        } else if cur == 0 {
                            // current is '='
                            break;
                        }
                    } else if start + cur + 1 == index.len() {
                        name_end = start + cur + 1;
                    }
                    if name_end > 0 {
                        let name = pattern.get_substr(start, name_end);

                        debug!("get name from '{:?}': '{}'", pattern, name);
                        data_keeper.name = Some(name);
                        index.set(name_end);
                        break;
                    }
                }
            }
            Self::Equal => {
                index.inc(1);
            }
            Self::Value => {
                if !index.is_end() {
                    // if we are here, the left chars is value
                    let value = pattern.get_substr(index.get(), index.len());

                    debug!("get value from {:?}: {}", pattern, value);
                    data_keeper.value = Some(value);
                    index.set(index.len());
                } else {
                    error!(?pattern, "syntax error! require an value after '='.");
                    return Err(Error::arg_missing_value(pattern.get_pattern()));
                }
            }
            Self::End => {
                return Ok(true);
            }
        }

        self.self_transition(index, pattern);

        self.parse(index, pattern, data_keeper)
    }
}

#[cfg(test)]
mod test {
    use ustr::Ustr;

    use crate::arg::parser::parse_argument;
    use crate::gstr;

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
                ("--=xar", Some((Some("--"), Some(""), Some("xar"), false))),
                ("-foo=", None),
            ];

            let prefixs = vec![gstr("--"), gstr("-"), gstr("")];

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &prefixs, case.1);
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
                ("a", Some((Some(""), Some("a"), None, false))),
                ("/a", Some((Some(""), Some("a"), None, true))),
                ("a=b", Some((Some(""), Some("a"), Some("b"), false))),
                ("foo", Some((Some(""), Some("foo"), None, false))),
                ("/foo", Some((Some(""), Some("foo"), None, true))),
                ("foo=bar", Some((Some(""), Some("foo"), Some("bar"), false))),
                ("--=xar", Some((Some("--"), Some(""), Some("xar"), false))),
                ("-foo=", None),
            ];

            let prefixs = vec![gstr("--"), gstr("-")];

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &prefixs, case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Ustr,
        prefix: &Vec<Ustr>,
        except: Option<(Option<&str>, Option<&str>, Option<&str>, bool)>,
    ) {
        let ret = parse_argument(pattern, prefix);

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
            dbg!(&except);
            dbg!(&ret);
            assert!(except.is_none());
        }
    }
}
