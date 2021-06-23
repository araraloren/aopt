
use crate::str::Str;
use crate::err::{Result, Error};
use crate::pattern::{ParseIndex, ParserPattern};

pub fn parse_argument<'pat, 'vec, 'pre>(pattern: &'pat str, prefix: &'vec Vec<Str<'pre>>) -> Result<DataKeeper<'pat, 'vec>> {
    let pattern = ParserPattern::new(pattern, prefix);
    let mut index = ParseIndex::new(pattern.len());
    let mut data_keeper = DataKeeper::default();

    let res = State::default().parse(&mut index, &pattern, &mut data_keeper)?;

    if res {
        debug!("With pattern: {:?}, parse result -> {:?}", pattern.get_pattern(), data_keeper);
        // must have prefix and name, prefix can be null string `""`
        if data_keeper.prefix.is_some() {
            if data_keeper.name.is_some() {
                return Ok(data_keeper);
            }
        }        
    }
    
    Err(Error::InvalidArgAsOption(String::from(pattern.get_pattern())))
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
pub struct DataKeeper<'nv, 'p> {
    pub name: Option<Str<'nv>>,

    pub value: Option<Str<'nv>>,

    pub prefix: Option<Str<'p>>,

    pub disable: bool,
}

impl<'nv, 'p> DataKeeper<'nv, 'p> {
    pub fn check_valid(&self) -> Result<bool> {
        todo!()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::PreCheck
    }
}

impl State {
    pub fn self_transition<'pat, 'vec, 'pre>(&mut self, index: &ParseIndex, pattern: &ParserPattern<'pat, 'vec, 'pre>) {
        let mut next_state = Self::End;

        match self.clone() {
            Self::PreCheck => {
                next_state = Self::Prefix;
            }
            Self::Prefix => {
                if let Some(ch) = pattern.left_chars(index.get()).nth(0) {
                    // match the deactivate char
                    next_state = if ch == '/' { Self::Disable } else { Self::Name };
                }
            }
            Self::Disable => {
                next_state = Self::Name;
            }
            Self::Name => {
                if let Some(ch) = pattern.left_chars(index.get()).nth(0) {
                    // match the equal char
                    next_state =  if ch == '=' { Self::Equal } else { Self::End }
                }
            }
            Self::Equal => {
                next_state = Self::Value
            }
            Self::Value => {
                next_state = Self::End
            }
            Self::End => {
                unreachable!("The end state can't going on!");
            }
        }

        debug!("Transition from {:?} --to--> {:?}", self, next_state);

        *self = next_state
    }

    pub fn parse<'pat, 'vec, 'pre>(
        mut self,
        index: &mut ParseIndex,
        pattern: & ParserPattern<'pat, 'vec, 'pre>,
        data_keeper: &mut DataKeeper<'pat, 'vec>,
    ) -> Result<bool> {
        if self != Self::End {
            debug!("Current state = {:?}, {:?}, parse pattern = {:?}", self, index, pattern);
            
            self.self_transition(index, pattern);

            let next_state = self.clone();

            match next_state {
                Self::Prefix => {
                    for prefix in pattern.get_prefixs() {
                        if pattern.get_pattern().starts_with(prefix.as_ref()) {
                            data_keeper.prefix = Some(Str::borrowed(prefix.as_ref()));
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
                    let mut temp_index = index.get();
                    let start = temp_index;

                    // get the chars until we meet '=' or reach the end
                    for ch in pattern.left_chars(temp_index) {
                        temp_index += 1;
                        if ch == '=' {
                            // the name not include '=', so > 1
                            if temp_index - start > 1 {
                                data_keeper.name = Some(Str::borrowed(
                                    pattern.get_pattern()
                                                .get(start .. temp_index - 1)
                                                .ok_or(Error::InvalidStrRange { beg: start, end: temp_index - 1 })?
                                ));
                                index.set(temp_index - 1);
                            }
                            break;
                        }
                        else if temp_index == index.len() {
                            // all the chars if name
                            if temp_index - start >= 1 {
                                data_keeper.name = Some(Str::borrowed(
                                    pattern.get_pattern()
                                                 .get(start .. temp_index)
                                                 .ok_or(Error::InvalidStrRange { beg: start, end: temp_index })?
                                ));
                                index.set(temp_index);
                            }
                            break;
                        }
                    }
                }
                Self::Equal => {
                    index.inc(1);
                }
                Self::Value => {
                    if ! index.is_end() {
                        // if we are here, the left chars is value
                        data_keeper.value = Some(Str::borrowed(
                            pattern.get_pattern()
                                        .get(index.get() ..)
                                        .ok_or(Error::InvalidStrRange { beg: index.get(), end: index.len() })?
                        ));
                        index.set(index.len());
                    }
                    else {
                        return Err(Error::InvalidArgArgument(String::from(pattern.get_pattern())));
                    }
                }
                _ => { }
            }

            next_state.parse(index, pattern, data_keeper)
        }
        else {
            Ok(true)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::str::Str;
    use crate::arg::parser::parse_argument;

    #[test]
    fn test_for_input_parser() {
        let test_cases = vec![
            ("", None),

            ("-a", Some((Some("-"), Some("a"), None, false))),
            ("-/a", Some((Some("-"), Some("a"), None, true))),
            ("-a=b", Some((Some("-"), Some("a"), Some("b"), false))),

            ("--foo", Some((Some("--"), Some("foo"), None, false))),
            ("--/foo", Some((Some("--"), Some("foo"), None, true))),
            ("--foo=bar", Some((Some("--"), Some("foo"), Some("bar"), false))),

            ("a", Some((Some(""), Some("a"), None, false))),
            ("/a", Some((Some(""), Some("a"), None, true))),
            ("a=b", Some((Some(""), Some("a"), Some("b"), false))),

            ("foo", Some((Some(""), Some("foo"), None, false))),
            ("/foo", Some((Some(""), Some("foo"), None, true))),
            ("foo=bar", Some((Some(""), Some("foo"), Some("bar"), false))),

            ("--=bar", None),
            ("-foo=", None),
        ];

        let prefixs = vec![
            Str::borrowed("--"),
            Str::borrowed("-"),
            Str::borrowed(""),
        ];

        for case in test_cases.iter() {
            try_to_verify_one_task(case.0, &prefixs,case.1);
        }
    }

    fn try_to_verify_one_task(pattern: &str, prefix: &Vec<Str<'_>>, except: Option<(Option<&str>, Option<&str>, Option<&str>, bool)>) {
        let ret = parse_argument(pattern, prefix);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            let default = Str::borrowed("");

            if let Some(except) = except {
                assert_eq!(except.0.unwrap_or(""), dk.prefix.unwrap_or(default.clone()).as_ref());
                assert_eq!(except.1.unwrap_or(""), dk.name.unwrap_or(default.clone()).as_ref());
                assert_eq!(except.2.unwrap_or(""), dk.value.unwrap_or(default.clone()).as_ref());
                assert_eq!(except.3, dk.disable);
            }
        }
        else {
            assert!(except.is_none());
        }
    }
}