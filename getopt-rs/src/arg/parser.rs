#![feature(try_blocks)]

use crate::err::{ArgumentError, Error, Result};
use crate::pat::{ParseIndex, ParserPattern};

pub fn parse_argument<'pre>(pattern: &str, prefix: &'pre [String]) -> Result<DataKeeper<'pre>> {
    let pattern = ParserPattern::new(pattern, prefix);
    let mut index = ParseIndex::new(pattern.len());
    let mut data_keeper = DataKeeper::default();

    let res = State::default().parse(&mut index, &pattern, &mut data_keeper)?;

    if res {
        tracing::debug!(?pattern, %prefix, ?data_keeper, "parsing argument successed");
        return Ok(data_keeper);
    }
    tracing::error!(?pattern, %prefix, ?index, "parsing argument failed");
    Err(ArgumentError::ParsingFailed(
        pattern.get_pattern().to_owned(),
    ));
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
pub struct DataKeeper<'pre> {
    pub name: Option<String>,

    pub value: Option<String>,

    pub prefix: Option<&'pre String>,

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
    #[tracing::instrument]
    pub fn self_transition<'pat, 'pre>(
        &mut self,
        index: &ParseIndex,
        pattern: &ParserPattern<'pat, 'pre>,
    ) {
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
        tracing::debug!("transition state from '{}' to '{}'", self, next_state);
        *self = next_state;
    }

    #[tracing::instrument]
    pub fn parse<'pat, 'pre>(
        mut self,
        index: &mut ParseIndex,
        pattern: &ParserPattern<'pat, 'pre>,
        data_keeper: &mut DataKeeper<'pre>,
    ) -> Result<bool> {
        let current_state = self.clone();

        match current_state {
            Self::PreCheck => {
                if pattern.get_pattern().is_empty() {
                    tracing::warn!("got an empty pattern");
                    return Ok(false);
                }
            }
            Self::Prefix => {
                for prefix in pattern.get_prefixs() {
                    if pattern.get_pattern().starts_with(prefix) {
                        data_keeper.prefix = Some(&prefix);
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
                for (cur, ch) in pattern.chars(end).enumerate() {
                    let name_end = 0;
                    // the name not include '=', so > 1
                    if ch == VALUE_SPLIT_CHAR && cur > start {
                        name_end = cur;
                    } else if end == index.len() && cur >= start {
                        name_end = cur + 1;
                    }
                    if name_end > 0 {
                        let name = pattern.get_pattern().get(start..name_end);

                        if name.is_none() {
                            tracing::error!(
                                ?pattern,
                                "accessing string [{}, {}) failed",
                                start,
                                real_end
                            );
                            return Err(ArgumentError::PatternAccessFailed(
                                pattern.get_pattern().to_owned(),
                                start,
                                name_end,
                            ));
                        }
                        data_keeper.name = name.to_owned();
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
                    let value = pattern.get_pattern().get(index.get()..);

                    if value.is_none() {
                        tracing::error!(
                            ?pattern,
                            "accessing string [{}, {}) failed",
                            index.get(),
                            index.len()
                        );
                        return Err(ArgumentError::PatternAccessFailed(
                            pattern.get_pattern().to_owned(),
                            index.get(),
                            index.len(),
                        ));
                    }
                    data_keeper.value = value.to_owned();
                    index.set(index.len());
                } else {
                    tracing::error!(?pattern, "syntax error! require an value after '='.");
                    return Err(ArgumentError::RequireValueForArgument(
                        pattern.get_pattern().to_owned(),
                    ));
                }
            }
            Self::End => {
                return Ok(true);
            }
            _ => {}
        }

        self.self_transition(index, pattern);

        self.parse(index, pattern, data_keeper)
    }
}

#[cfg(test)]
mod test {
    use crate::arg::parser::parse_argument;

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

            let prefixs = vec![String::from("--"), String::from("-"), String::from("")];

            for case in test_cases.iter() {
                try_to_verify_one_task(case.0, &prefixs, case.1);
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

            let prefixs = vec![String::from("--"), String::from("-")];

            for case in test_cases.iter() {
                try_to_verify_one_task(case.0, &prefixs, case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: &str,
        prefix: &Vec<String>,
        except: Option<(Option<&str>, Option<&str>, Option<&str>, bool)>,
    ) {
        let ret = parse_argument(pattern, prefix);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            if except.is_none() {
                panic!("----> {:?}", except);
            }

            let default = String::from("");

            if let Some(except) = except {
                assert_eq!(except.0.unwrap_or(""), dk.prefix.unwrap_or(&default));
                assert_eq!(except.1.unwrap_or(""), dk.name.unwrap_or(default.clone()));
                assert_eq!(except.2.unwrap_or(""), dk.value.unwrap_or(default.clone()));
                assert_eq!(except.3, dk.disable);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
