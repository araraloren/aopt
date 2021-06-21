
use crate::str::Str;
use crate::err::{Result, Error};
use crate::pattern::{ParseIndex, ParserPattern};

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
        *self = next_state
    }

    pub fn parse<'pat, 'vec, 'pre>(
        mut self,
        index: &mut ParseIndex,
        pattern: & ParserPattern<'pat, 'vec, 'pre>,
        data_keeper: &mut DataKeeper<'pat, 'vec>,
    ) -> Result<bool> {
        let state = self.clone();

        if index.is_end() && state != Self::End {
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

                    for ch in pattern.left_chars(temp_index) {
                        temp_index += 1;
                        if ch == '=' {
                            data_keeper.name = Some(Str::borrowed(
                                pattern.get_pattern()
                                             .get(start .. temp_index - 1)
                                             .ok_or(Error::InvalidStrRange { beg: start, end: temp_index - 1 })?
                            ));
                            index.set(temp_index - 1);
                        }
                        else if temp_index == index.len() {
                            if temp_index - start > 1 {
                                data_keeper.name = Some(Str::borrowed(
                                    pattern.get_pattern()
                                                 .get(start .. temp_index)
                                                 .ok_or(Error::InvalidStrRange { beg: start, end: temp_index })?
                                ));
                                index.set(temp_index);
                            }
                        }
                    }
                }
                Self::Equal => {
                    index.inc(1);
                }
                Self::Value => {
                    data_keeper.value = Some(Str::borrowed(
                        pattern.get_pattern()
                                     .get(index.get() ..)
                                     .ok_or(Error::InvalidStrRange { beg: index.get(), end: index.len() })?
                    ));
                    index.set(index.len());
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
    #[test]
    fn test_for_input_parser() {

    }
}