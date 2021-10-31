use crate::err::{ArgumentError, Result};
use crate::pat::{ParseIndex, ParserPattern};
use crate::OptStr;

pub fn parse_argument(pattern: OptStr, prefix: &[OptStr]) -> Result<DataKeeper> {
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
    Err(ArgumentError::ParsingFailed(pattern.get_pattern().to_owned()).into())
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
    pub name: Option<OptStr>,

    pub value: Option<OptStr>,

    pub prefix: Option<OptStr>,

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
                if pattern.get_pattern().is_empty() {
                    warn!("got an empty pattern");
                    return Ok(false);
                }
            }
            Self::Prefix => {
                for prefix in pattern.get_prefixs() {
                    if pattern.get_pattern().starts_with(prefix.as_ref()) {
                        data_keeper.prefix = Some(prefix.clone());
                        index.inc(prefix.len());
                        break;
                    }
                }
            }
            Self::Disable => {
                data_keeper.disable = true;
                index.inc(1);
            }
            Self::Name => {
                let start = index.get();

                // get the chars until we meet '=' or reach the end
                for (cur, ch) in pattern.chars(start).enumerate() {
                    let mut name_end = 0;
                    // the name not include '=', so > 1
                    if ch == VALUE_SPLIT_CHAR && cur >= 1 {
                        name_end = start + cur;
                    } else if start + cur + 1 == index.len() {
                        name_end = start + cur + 1;
                    }
                    if name_end > 0 {
                        let name = pattern.get_pattern().get(start..name_end);

                        if let Some(name) = name {
                            data_keeper.name = Some(name.into());
                            index.set(name_end);
                        } else {
                            error!(
                                ?pattern,
                                "accessing string [{}, {}) failed", start, name_end
                            );
                            return Err(ArgumentError::PatternAccessFailed(
                                pattern.get_pattern().to_owned(),
                                start,
                                name_end,
                            )
                            .into());
                        }
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
                    let value = pattern.get_pattern().get(index.get()..);

                    if let Some(value) = value {
                        data_keeper.value = Some(value.into());
                        index.set(index.len());
                    } else {
                        error!(
                            ?pattern,
                            "accessing string [{}, {}) failed",
                            index.get(),
                            index.len()
                        );
                        return Err(ArgumentError::PatternAccessFailed(
                            pattern.get_pattern().to_owned(),
                            index.get(),
                            index.len(),
                        )
                        .into());
                    }
                } else {
                    error!(?pattern, "syntax error! require an value after '='.");
                    return Err(
                        ArgumentError::MissingValue(pattern.get_pattern().to_owned()).into(),
                    );
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
    use crate::arg::parser::parse_argument;
    use crate::OptStr;

    #[test]
    fn test_for_input_parser() {
        {
            // test 1
            let test_cases = vec![
                ("", Some((Some(""), Some(""), None, false))),
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
                ("--=bar", Some((Some("--"), Some(""), Some("bar"), false))),
                ("-foo=", None),
            ];

            let prefixs = vec![OptStr::from("--"), OptStr::from("-"), OptStr::from("")];

            for case in test_cases.iter() {
                try_to_verify_one_task(OptStr::from(case.0), &prefixs, case.1);
            }
        }
        {
            // test 2
            let test_cases = vec![
                ("", Some((Some(""), Some(""), None, false))),
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
                ("--=bar", Some((Some("--"), Some(""), Some("bar"), false))),
                ("-foo=", None),
            ];

            let prefixs = vec![OptStr::from("--"), OptStr::from("-")];

            for case in test_cases.iter() {
                try_to_verify_one_task(OptStr::from(case.0), &prefixs, case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: OptStr,
        prefix: &Vec<OptStr>,
        except: Option<(Option<&str>, Option<&str>, Option<&str>, bool)>,
    ) {
        let ret = parse_argument(pattern, prefix);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            if except.is_none() {
                panic!("----> {:?}", except);
            }

            let default = OptStr::from("");

            if let Some(except) = except {
                assert_eq!(
                    except.0.unwrap_or(""),
                    dk.prefix.unwrap_or(default).as_ref()
                );
                assert_eq!(
                    except.1.unwrap_or(""),
                    dk.name.unwrap_or(default.clone()).as_ref()
                );
                assert_eq!(
                    except.2.unwrap_or(""),
                    dk.value.unwrap_or(default.clone()).as_ref()
                );
                assert_eq!(except.3, dk.disable);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
