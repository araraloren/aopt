
use crate::err::Result;
use crate::err::Error;
use crate::opt::index::Index;
use crate::pat::{ParserPattern, ParseIndex};

pub fn parse_option_str<'pre>(pattern: &str, prefix: &'pre Vec<String>) -> Result<DataKeeper<'pre>> {
    let pattern = ParserPattern::new(pattern, prefix);
    let mut index = ParseIndex::new(pattern.len());
    let mut data_keeper = DataKeeper::default();

    let res = State::default().parse(&mut index, &pattern, &mut data_keeper)?;

    if res {
        debug!("With pattern: {:?}, parse result -> {:?}", pattern.get_pattern(), data_keeper);
        // don't check anything 
        return Ok(data_keeper);
    }
    
    Err(Error::InvalidOptionStr(String::from(pattern.get_pattern())))
}
    
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum State {
    PreCheck,
    Prefix,
    Name,
    Equal,
    Type,
    Deactivate,
    Optional,
    Index,
    FowradIndex,
    BackwardIndex,
    List,
    Except,
    End,
}

#[derive(Debug, Default)]
pub struct DataKeeper<'pre> {
    pub prefix: Option<&'pre String>,

    pub name: Option<String>,

    pub type_name: Option<String>,

    pub deactivate: bool,

    pub optional: bool,

    pub forward_index: Option<u64>,

    pub backward_index: Option<u64>,

    pub anywhere: Option<bool>,

    pub list: Vec<u64>,

    pub except: Vec<u64>,
}

impl<'pre> DataKeeper<'pre> {
    pub fn gen_index(&mut self) -> Index {
        if self.forward_index.is_some() {
            Index::forward(self.forward_index.unwrap())
        }
        else if self.backward_index.is_some() {
            Index::backward(self.backward_index.unwrap())
        }
        else if self.anywhere.unwrap_or(false) {
            Index::anywhere()
        }
        else if self.list.len() > 0 {
            Index::list(std::mem::take(&mut self.list))
        }
        else if self.except.len() > 0 {
            Index::except(std::mem::take(&mut self.except))
        }
        else {
            Index::default()
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::PreCheck
    }
}

impl<'a, 'b, 'c> State {
    pub fn self_transition<'pat, 'vec, 'pre>(&mut self, index: &ParseIndex, pattern: &ParserPattern<'pat, 'pre>) {
        let mut next_state = Self::End;

        match self.clone() {
            Self::PreCheck => {
                next_state = Self::Prefix;
            }
            Self::Prefix => {
                if let Some(_) = pattern.left_chars(index.get()).nth(0) {
                    next_state = Self::Name
                }
            }
            Self::Name => {
                if let Some(ch) = pattern.left_chars(index.get()).nth(0) {
                    next_state = match ch {
                        // equal state will increment the index
                        '=' => Self::Equal,
                        _ => Self::Type,
                    };
                }
            }
            Self::Equal => {
                next_state = Self::Type;
            }
            State::Type | State::Deactivate | State::Optional => {
                if let Some(ch) = pattern.left_chars(index.get()).nth(0) {
                    next_state = match ch {
                        '!' => Self::Optional,
                        '/' => Self::Deactivate,
                        '@' => Self::Index,
                        _ => Self::End,
                    };
                }
            }
            State::Index => {
                let (_, index_part) = pattern.get_pattern().split_at(index.get());

                next_state = if index_part.starts_with("+[") || index_part.starts_with("[") {
                    State::List
                } else if index_part.starts_with("-[") {
                    State::Except
                } else if index_part.starts_with("-") {
                    State::BackwardIndex
                } else {
                    State::FowradIndex
                };
            }
            State::FowradIndex | State::BackwardIndex | State::List | State::Except => { },
            State::End => {
                unreachable!("The end state can't going on!");
            },
        }

        debug!("Transition from {:?} --to--> {:?}", self, next_state);

        *self = next_state;
    }

    pub fn parse<'pat, 'pre>(
        mut self,
        index: &mut ParseIndex,
        pattern: & ParserPattern<'pat, 'pre>,
        data_keeper: &mut DataKeeper<'pre>,
    ) -> Result<bool> {
        if self != State::End {
            debug!("Current state = {:?}, {:?}, parse pattern = {:?}", self, index, pattern);

            self.self_transition(index, pattern);

            let next_state = self.clone();

            match next_state {
                State::Prefix => {
                    for prefix in pattern.get_prefixs() {
                        if pattern.get_pattern().starts_with(prefix) {
                            data_keeper.prefix = Some(&prefix);
                            index.inc(prefix.len());
                            break;
                        }
                    }
                }
                State::Name => {
                    let mut cur_index = index.get();
                    let start = cur_index;

                    for ch in pattern.left_chars(cur_index) {
                        cur_index += 1;
                        if ch == '=' || ch == '!' || ch == '/' || ch == '@' {
                            if cur_index - start > 1 {
                                data_keeper.name = Some(
                                    pattern.get_pattern()
                                        .get(start .. cur_index - 1)
                                        .ok_or(Error::InvalidStrRange { beg: start, end: cur_index - 1})?
                                        .to_owned()
                                );
                                index.set(cur_index - 1);
                            }
                            break;
                        } else if cur_index == index.len() {
                            if cur_index - start >= 1 {
                                data_keeper.name = Some(
                                    pattern.get_pattern()
                                        .get(start .. cur_index)
                                        .ok_or(Error::InvalidStrRange { beg: start, end: cur_index})?
                                        .to_owned()
                                );
                                index.set(cur_index);
                            }
                            break;
                        }
                    }
                }
                State::Equal => {
                    index.inc(1);
                }
                State::Type => {
                    let mut cur_index = index.get();
                    let start = cur_index;

                    for ch in pattern.left_chars(cur_index) {
                        cur_index += 1;
                        if ch == '!' || ch == '/' || ch == '@' {
                            if cur_index - start > 1 {
                                data_keeper.type_name = Some(
                                    pattern.get_pattern()
                                        .get(start .. cur_index - 1)
                                        .ok_or(Error::InvalidStrRange { beg: start, end: cur_index - 1 })?
                                        .to_owned()
                                );
                                index.set(cur_index - 1);
                            }
                            break;
                        } else if cur_index == index.len() {
                            if cur_index - start >= 1 {
                                data_keeper.type_name = Some(
                                    pattern.get_pattern()
                                        .get(start .. cur_index)
                                        .ok_or(Error::InvalidStrRange { beg: start, end: cur_index})?
                                        .to_owned()
                                );
                                index.set(cur_index);
                            }
                            break;
                        }
                    }
                }
                State::Deactivate => {
                    data_keeper.deactivate = true;
                    index.inc(1);
                }
                State::Optional => {
                    data_keeper.optional = true;
                    index.inc(1);
                }
                State::Index => {
                    index.inc(1);
                }
                State::FowradIndex => {
                    let (_, index_part) = pattern.get_pattern().split_at(index.get());

                    let ret = index_part.parse::<u64>()
                                                    .map_err(|e| Error::InavlidOptionIndexValue(format!("{:?}", e)))?;
                    if ret > 0 {
                        data_keeper.forward_index = Some(ret);
                    }
                    else {
                        data_keeper.anywhere = Some(true);
                    }
                    index.set(index.len());
                }
                State::BackwardIndex => {
                    let (_, index_part) = pattern.get_pattern().split_at(index.get() + 1);

                    let ret = index_part.parse::<u64>()
                                                    .map_err(|e| Error::InavlidOptionIndexValue(format!("{:?}", e)))?;
                    if ret > 0 {
                        data_keeper.backward_index = Some(ret);
                    }
                    else {
                        data_keeper.anywhere = Some(true);
                    }
                    index.set(index.len());
                }
                State::List => {
                    let (_, index_part) = pattern.get_pattern().split_at(index.get());

                    if index_part.starts_with("+[") {
                        let index_part = pattern
                            .get_pattern()
                            .get(index.get() + 2 .. index.len() - 1)
                            .unwrap();

                        data_keeper.list = index_part
                            .split(',')
                            .map(|v| v.trim().parse::<u64>().map_err(|e| Error::InavlidOptionIndexValue(format!("{:?}", e))))
                            .collect::<Result<Vec<u64>>>()?;
                    } else {
                        let index_part = pattern
                            .get_pattern()
                            .get(index.get() + 1 .. index.len() - 1)
                            .unwrap();

                        data_keeper.list = index_part
                            .split(',')
                            .map(|v| v.trim().parse::<u64>().map_err(|e| Error::InavlidOptionIndexValue(format!("{:?}", e))))
                            .collect::<Result<Vec<u64>>>()?;
                    }
                    index.set(index.len());
                }
                State::Except => {
                    let index_part = pattern
                        .get_pattern()
                        .get(index.get() + 2 .. index.len() - 1)
                        .unwrap();

                    data_keeper.except = index_part
                        .split(',')
                        .map(|v| v.trim().parse::<u64>().map_err(|e| Error::InavlidOptionIndexValue(format!("{:?}", e))))
                        .collect::<Result<Vec<u64>>>()?;
                    index.set(index.len());
                }
                State::End => {
                    if !index.is_end() {
                        return Err(Error::InvalidOptionStr(format!("{}", pattern.get_pattern())));
                    }
                }
                _ => {}
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
    use crate::set::parser::parse_option_str;
    use crate::opt::index::Index;

    #[test]
    fn test_for_input_parser() {
        {// test 1
            let test_cases = vec![
                ("", Some((None, None, None, Index::default(), false, true))),
                ("o=b", Some((None, Some("o"), Some("b"), Index::default(), false, true))),
                ("o=b!", Some((None, Some("o"), Some("b"), Index::default(), false, false))),
                ("o=b/", Some((None, Some("o"), Some("b"), Index::default(), true, true))),
                ("o=b!/", Some((None, Some("o"), Some("b"), Index::default(), true, false))),
                ("o=b/!", Some((None, Some("o"), Some("b"), Index::default(), true, false))),
                ("-o=b", Some((Some("-"), Some("o"), Some("b"), Index::default(), false, true))),
                ("-o=b!", Some((Some("-"), Some("o"), Some("b"), Index::default(), false, false))),
                ("-o=b/", Some((Some("-"), Some("o"), Some("b"), Index::default(), true, true))),
                ("-o=b!/", Some((Some("-"), Some("o"), Some("b"), Index::default(), true, false))),
                ("-o=b/!", Some((Some("-"), Some("o"), Some("b"), Index::default(), true, false))),
                ("--o=b", Some((Some("--"), Some("o"), Some("b"), Index::default(), false, true))),
                ("--o=b!", Some((Some("--"), Some("o"), Some("b"), Index::default(), false, false))),
                ("--o=b/", Some((Some("--"), Some("o"), Some("b"), Index::default(), true, true))),
                ("--o=b!/", Some((Some("--"), Some("o"), Some("b"), Index::default(), true, false))),
                ("--o=b/!", Some((Some("--"), Some("o"), Some("b"), Index::default(), true, false))),
                ("=b", Some((None, None, Some("b"), Index::default(), false, true))),
                ("=b!", Some((None, None, Some("b"), Index::default(), false, false))),
                ("=b/", Some((None, None, Some("b"), Index::default(), true, true))),
                ("=b!/", Some((None, None, Some("b"), Index::default(), true, false))),
                ("=b/!", Some((None, None, Some("b"), Index::default(), true, false))),
                ("o=b@1", Some((None, Some("o"), Some("b"), Index::forward(1), false, true))),
                ("o=b@-1", Some((None, Some("o"), Some("b"), Index::backward(1), false, true))),
                ("o=b@+42", Some((None, Some("o"), Some("b"), Index::forward(42), false, true))),
                ("o=b@[1, 2, 3]", Some((None, Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, true))),
                ("o=b@+[4, 5, 12]", Some((None, Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, true))),
                ("o=b@-[1, 2, 4]", Some((None, Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, true))),
                ("-o=b@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), false, true))),
                ("-o=b@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), false, true))),
                ("-o=b@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), false, true))),
                ("-o=b@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, true))),
                ("-o=b@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, true))),
                ("-o=b@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, true))),
                ("--o=b@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), false, true))),
                ("--o=b@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), false, true))),
                ("--o=b@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), false, true))),
                ("--o=b@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, true))),
                ("--o=b@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, true))),
                ("--o=b@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, true))),
                ("o=b!@1", Some((None, Some("o"), Some("b"), Index::forward(1), false, false))),
                ("o=b!@-1", Some((None, Some("o"), Some("b"), Index::backward(1), false, false))),
                ("o=b!@+42", Some((None, Some("o"), Some("b"), Index::forward(42), false, false))),
                ("o=b!@[1, 2, 3]", Some((None, Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, false))),
                ("o=b!@+[4, 5, 12]", Some((None, Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, false))),
                ("o=b!@-[1, 2, 4]", Some((None, Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, false))),
                ("-o=b!@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), false, false))),
                ("-o=b!@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), false, false))),
                ("-o=b!@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), false, false))),
                ("-o=b!@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, false))),
                ("-o=b!@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, false))),
                ("-o=b!@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, false))),
                ("--o=b!@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), false, false))),
                ("--o=b!@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), false, false))),
                ("--o=b!@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), false, false))),
                ("--o=b!@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, false))),
                ("--o=b!@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, false))),
                ("--o=b!@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, false))),
                ("o=b/@1", Some((None, Some("o"), Some("b"), Index::forward(1), true, true))),
                ("o=b/@-1", Some((None, Some("o"), Some("b"), Index::backward(1), true, true))),
                ("o=b/@+42", Some((None, Some("o"), Some("b"), Index::forward(42), true, true))),
                ("o=b/@[1, 2, 3]", Some((None, Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, true))),
                ("o=b/@+[4, 5, 12]", Some((None, Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, true))),
                ("o=b/@-[1, 2, 4]", Some((None, Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, true))),
                ("-o=b/@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), true, true))),
                ("-o=b/@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), true, true))),
                ("-o=b/@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), true, true))),
                ("-o=b/@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, true))),
                ("-o=b/@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, true))),
                ("-o=b/@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, true))),
                ("--o=b/@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), true, true))),
                ("--o=b/@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), true, true))),
                ("--o=b/@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), true, true))),
                ("--o=b/@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, true))),
                ("--o=b/@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, true))),
                ("--o=b/@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, true))),
                ("o=b!/@1", Some((None, Some("o"), Some("b"), Index::forward(1), true, false))),
                ("o=b!/@-1", Some((None, Some("o"), Some("b"), Index::backward(1), true, false))),
                ("o=b!/@+42", Some((None, Some("o"), Some("b"), Index::forward(42), true, false))),
                ("o=b!/@[1, 2, 3]", Some((None, Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("o=b!/@+[4, 5, 12]", Some((None, Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("o=b!/@-[1, 2, 4]", Some((None, Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("-o=b!/@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("-o=b!/@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("-o=b!/@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("-o=b!/@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("-o=b!/@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("-o=b!/@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("--o=b!/@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("--o=b!/@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("--o=b!/@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("--o=b!/@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("--o=b!/@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("--o=b!/@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("o=b/!@1", Some((None, Some("o"), Some("b"), Index::forward(1), true, false))),
                ("o=b/!@-1", Some((None, Some("o"), Some("b"), Index::backward(1), true, false))),
                ("o=b/!@+42", Some((None, Some("o"), Some("b"), Index::forward(42), true, false))),
                ("o=b/!@[1, 2, 3]", Some((None, Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("o=b/!@+[4, 5, 12]", Some((None, Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("o=b/!@-[1, 2, 4]", Some((None, Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("-o=b/!@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("-o=b/!@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("-o=b/!@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("-o=b/!@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("-o=b/!@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("-o=b/!@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("--o=b/!@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("--o=b/!@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("--o=b/!@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("--o=b/!@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("--o=b/!@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("--o=b/!@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("option=bar", Some((None, Some("option"), Some("bar"), Index::default(), false, true))),
                ("option=bar!", Some((None, Some("option"), Some("bar"), Index::default(), false, false))),
                ("option=bar/", Some((None, Some("option"), Some("bar"), Index::default(), true, true))),
                ("option=bar!/", Some((None, Some("option"), Some("bar"), Index::default(), true, false))),
                ("option=bar/!", Some((None, Some("option"), Some("bar"), Index::default(), true, false))),
                ("-option=bar", Some((Some("-"), Some("option"), Some("bar"), Index::default(), false, true))),
                ("-option=bar!", Some((Some("-"), Some("option"), Some("bar"), Index::default(), false, false))),
                ("-option=bar/", Some((Some("-"), Some("option"), Some("bar"), Index::default(), true, true))),
                ("-option=bar!/", Some((Some("-"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("-option=bar/!", Some((Some("-"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("--option=bar", Some((Some("--"), Some("option"), Some("bar"), Index::default(), false, true))),
                ("--option=bar!", Some((Some("--"), Some("option"), Some("bar"), Index::default(), false, false))),
                ("--option=bar/", Some((Some("--"), Some("option"), Some("bar"), Index::default(), true, true))),
                ("--option=bar!/", Some((Some("--"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("--option=bar/!", Some((Some("--"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("=bar", Some((None, None, Some("bar"), Index::default(), false, true))),
                ("=bar!", Some((None, None, Some("bar"), Index::default(), false, false))),
                ("=bar/", Some((None, None, Some("bar"), Index::default(), true, true))),
                ("=bar!/", Some((None, None, Some("bar"), Index::default(), true, false))),
                ("=bar/!", Some((None, None, Some("bar"), Index::default(), true, false))),
                ("option=bar@1", Some((None, Some("option"), Some("bar"), Index::forward(1), false, true))),
                ("option=bar@-1", Some((None, Some("option"), Some("bar"), Index::backward(1), false, true))),
                ("option=bar@+42", Some((None, Some("option"), Some("bar"), Index::forward(42), false, true))),
                ("option=bar@[1, 2, 3]", Some((None, Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, true))),
                ("option=bar@+[4, 5, 12]", Some((None, Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, true))),
                ("option=bar@-[1, 2, 4]", Some((None, Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, true))),
                ("-option=bar@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), false, true))),
                ("-option=bar@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), false, true))),
                ("-option=bar@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), false, true))),
                ("-option=bar@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, true))),
                ("-option=bar@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, true))),
                ("-option=bar@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, true))),
                ("--option=bar@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), false, true))),
                ("--option=bar@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), false, true))),
                ("--option=bar@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), false, true))),
                ("--option=bar@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, true))),
                ("--option=bar@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, true))),
                ("--option=bar@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, true))),
                ("option=bar!@1", Some((None, Some("option"), Some("bar"), Index::forward(1), false, false))),
                ("option=bar!@-1", Some((None, Some("option"), Some("bar"), Index::backward(1), false, false))),
                ("option=bar!@+42", Some((None, Some("option"), Some("bar"), Index::forward(42), false, false))),
                ("option=bar!@[1, 2, 3]", Some((None, Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, false))),
                ("option=bar!@+[4, 5, 12]", Some((None, Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, false))),
                ("option=bar!@-[1, 2, 4]", Some((None, Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, false))),
                ("-option=bar!@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), false, false))),
                ("-option=bar!@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), false, false))),
                ("-option=bar!@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), false, false))),
                ("-option=bar!@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, false))),
                ("-option=bar!@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, false))),
                ("-option=bar!@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, false))),
                ("--option=bar!@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), false, false))),
                ("--option=bar!@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), false, false))),
                ("--option=bar!@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), false, false))),
                ("--option=bar!@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, false))),
                ("--option=bar!@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, false))),
                ("--option=bar!@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, false))),
                ("option=bar/@1", Some((None, Some("option"), Some("bar"), Index::forward(1), true, true))),
                ("option=bar/@-1", Some((None, Some("option"), Some("bar"), Index::backward(1), true, true))),
                ("option=bar/@+42", Some((None, Some("option"), Some("bar"), Index::forward(42), true, true))),
                ("option=bar/@[1, 2, 3]", Some((None, Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, true))),
                ("option=bar/@+[4, 5, 12]", Some((None, Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, true))),
                ("option=bar/@-[1, 2, 4]", Some((None, Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, true))),
                ("-option=bar/@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), true, true))),
                ("-option=bar/@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), true, true))),
                ("-option=bar/@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), true, true))),
                ("-option=bar/@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, true))),
                ("-option=bar/@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, true))),
                ("-option=bar/@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, true))),
                ("--option=bar/@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), true, true))),
                ("--option=bar/@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), true, true))),
                ("--option=bar/@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), true, true))),
                ("--option=bar/@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, true))),
                ("--option=bar/@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, true))),
                ("--option=bar/@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, true))),
                ("option=bar!/@1", Some((None, Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("option=bar!/@-1", Some((None, Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("option=bar!/@+42", Some((None, Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("option=bar!/@[1, 2, 3]", Some((None, Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("option=bar!/@+[4, 5, 12]", Some((None, Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("option=bar!/@-[1, 2, 4]", Some((None, Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("-option=bar!/@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("-option=bar!/@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("-option=bar!/@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("-option=bar!/@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("-option=bar!/@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("-option=bar!/@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("--option=bar!/@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("--option=bar!/@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("--option=bar!/@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("--option=bar!/@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("--option=bar!/@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("--option=bar!/@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("option=bar/!@1", Some((None, Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("option=bar/!@-1", Some((None, Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("option=bar/!@+42", Some((None, Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("option=bar/!@[1, 2, 3]", Some((None, Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("option=bar/!@+[4, 5, 12]", Some((None, Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("option=bar/!@-[1, 2, 4]", Some((None, Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("-option=bar/!@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("-option=bar/!@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("-option=bar/!@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("-option=bar/!@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("-option=bar/!@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("-option=bar/!@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("--option=bar/!@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("--option=bar/!@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("--option=bar/!@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("--option=bar/!@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("--option=bar/!@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("--option=bar/!@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
            ];

            let prefixs = vec![
                String::from("--"),
                String::from("-"),
            ];

            for case in test_cases.iter() {
                try_to_verify_one_task(case.0, &prefixs,&case.1);
            }
        }
        {// test 2
            let test_cases = vec![
                ("", Some((Some(""), None, None, Index::default(), false, true))),
                ("o=b", Some((Some(""), Some("o"), Some("b"), Index::default(), false, true))),
                ("o=b!", Some((Some(""), Some("o"), Some("b"), Index::default(), false, false))),
                ("o=b/", Some((Some(""), Some("o"), Some("b"), Index::default(), true, true))),
                ("o=b!/", Some((Some(""), Some("o"), Some("b"), Index::default(), true, false))),
                ("o=b/!", Some((Some(""), Some("o"), Some("b"), Index::default(), true, false))),
                ("-o=b", Some((Some("-"), Some("o"), Some("b"), Index::default(), false, true))),
                ("-o=b!", Some((Some("-"), Some("o"), Some("b"), Index::default(), false, false))),
                ("-o=b/", Some((Some("-"), Some("o"), Some("b"), Index::default(), true, true))),
                ("-o=b!/", Some((Some("-"), Some("o"), Some("b"), Index::default(), true, false))),
                ("-o=b/!", Some((Some("-"), Some("o"), Some("b"), Index::default(), true, false))),
                ("--o=b", Some((Some("--"), Some("o"), Some("b"), Index::default(), false, true))),
                ("--o=b!", Some((Some("--"), Some("o"), Some("b"), Index::default(), false, false))),
                ("--o=b/", Some((Some("--"), Some("o"), Some("b"), Index::default(), true, true))),
                ("--o=b!/", Some((Some("--"), Some("o"), Some("b"), Index::default(), true, false))),
                ("--o=b/!", Some((Some("--"), Some("o"), Some("b"), Index::default(), true, false))),
                ("=b", Some((Some(""), None, Some("b"), Index::default(), false, true))),
                ("=b!", Some((Some(""), None, Some("b"), Index::default(), false, false))),
                ("=b/", Some((Some(""), None, Some("b"), Index::default(), true, true))),
                ("=b!/", Some((Some(""), None, Some("b"), Index::default(), true, false))),
                ("=b/!", Some((Some(""), None, Some("b"), Index::default(), true, false))),
                ("o=b@1", Some((Some(""), Some("o"), Some("b"), Index::forward(1), false, true))),
                ("o=b@-1", Some((Some(""), Some("o"), Some("b"), Index::backward(1), false, true))),
                ("o=b@+42", Some((Some(""), Some("o"), Some("b"), Index::forward(42), false, true))),
                ("o=b@[1, 2, 3]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, true))),
                ("o=b@+[4, 5, 12]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, true))),
                ("o=b@-[1, 2, 4]", Some((Some(""), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, true))),
                ("-o=b@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), false, true))),
                ("-o=b@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), false, true))),
                ("-o=b@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), false, true))),
                ("-o=b@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, true))),
                ("-o=b@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, true))),
                ("-o=b@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, true))),
                ("--o=b@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), false, true))),
                ("--o=b@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), false, true))),
                ("--o=b@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), false, true))),
                ("--o=b@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, true))),
                ("--o=b@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, true))),
                ("--o=b@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, true))),
                ("o=b!@1", Some((Some(""), Some("o"), Some("b"), Index::forward(1), false, false))),
                ("o=b!@-1", Some((Some(""), Some("o"), Some("b"), Index::backward(1), false, false))),
                ("o=b!@+42", Some((Some(""), Some("o"), Some("b"), Index::forward(42), false, false))),
                ("o=b!@[1, 2, 3]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, false))),
                ("o=b!@+[4, 5, 12]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, false))),
                ("o=b!@-[1, 2, 4]", Some((Some(""), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, false))),
                ("-o=b!@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), false, false))),
                ("-o=b!@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), false, false))),
                ("-o=b!@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), false, false))),
                ("-o=b!@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, false))),
                ("-o=b!@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, false))),
                ("-o=b!@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, false))),
                ("--o=b!@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), false, false))),
                ("--o=b!@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), false, false))),
                ("--o=b!@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), false, false))),
                ("--o=b!@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), false, false))),
                ("--o=b!@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), false, false))),
                ("--o=b!@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), false, false))),
                ("o=b/@1", Some((Some(""), Some("o"), Some("b"), Index::forward(1), true, true))),
                ("o=b/@-1", Some((Some(""), Some("o"), Some("b"), Index::backward(1), true, true))),
                ("o=b/@+42", Some((Some(""), Some("o"), Some("b"), Index::forward(42), true, true))),
                ("o=b/@[1, 2, 3]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, true))),
                ("o=b/@+[4, 5, 12]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, true))),
                ("o=b/@-[1, 2, 4]", Some((Some(""), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, true))),
                ("-o=b/@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), true, true))),
                ("-o=b/@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), true, true))),
                ("-o=b/@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), true, true))),
                ("-o=b/@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, true))),
                ("-o=b/@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, true))),
                ("-o=b/@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, true))),
                ("--o=b/@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), true, true))),
                ("--o=b/@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), true, true))),
                ("--o=b/@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), true, true))),
                ("--o=b/@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, true))),
                ("--o=b/@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, true))),
                ("--o=b/@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, true))),
                ("o=b!/@1", Some((Some(""), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("o=b!/@-1", Some((Some(""), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("o=b!/@+42", Some((Some(""), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("o=b!/@[1, 2, 3]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("o=b!/@+[4, 5, 12]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("o=b!/@-[1, 2, 4]", Some((Some(""), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("-o=b!/@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("-o=b!/@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("-o=b!/@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("-o=b!/@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("-o=b!/@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("-o=b!/@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("--o=b!/@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("--o=b!/@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("--o=b!/@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("--o=b!/@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("--o=b!/@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("--o=b!/@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("o=b/!@1", Some((Some(""), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("o=b/!@-1", Some((Some(""), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("o=b/!@+42", Some((Some(""), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("o=b/!@[1, 2, 3]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("o=b/!@+[4, 5, 12]", Some((Some(""), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("o=b/!@-[1, 2, 4]", Some((Some(""), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("-o=b/!@1", Some((Some("-"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("-o=b/!@-1", Some((Some("-"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("-o=b/!@+42", Some((Some("-"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("-o=b/!@[1, 2, 3]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("-o=b/!@+[4, 5, 12]", Some((Some("-"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("-o=b/!@-[1, 2, 4]", Some((Some("-"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("--o=b/!@1", Some((Some("--"), Some("o"), Some("b"), Index::forward(1), true, false))),
                ("--o=b/!@-1", Some((Some("--"), Some("o"), Some("b"), Index::backward(1), true, false))),
                ("--o=b/!@+42", Some((Some("--"), Some("o"), Some("b"), Index::forward(42), true, false))),
                ("--o=b/!@[1, 2, 3]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![1, 2, 3]), true, false))),
                ("--o=b/!@+[4, 5, 12]", Some((Some("--"), Some("o"), Some("b"), Index::list(vec![4, 5, 12]), true, false))),
                ("--o=b/!@-[1, 2, 4]", Some((Some("--"), Some("o"), Some("b"), Index::except(vec![1, 2, 4]), true, false))),
                ("option=bar", Some((Some(""), Some("option"), Some("bar"), Index::default(), false, true))),
                ("option=bar!", Some((Some(""), Some("option"), Some("bar"), Index::default(), false, false))),
                ("option=bar/", Some((Some(""), Some("option"), Some("bar"), Index::default(), true, true))),
                ("option=bar!/", Some((Some(""), Some("option"), Some("bar"), Index::default(), true, false))),
                ("option=bar/!", Some((Some(""), Some("option"), Some("bar"), Index::default(), true, false))),
                ("-option=bar", Some((Some("-"), Some("option"), Some("bar"), Index::default(), false, true))),
                ("-option=bar!", Some((Some("-"), Some("option"), Some("bar"), Index::default(), false, false))),
                ("-option=bar/", Some((Some("-"), Some("option"), Some("bar"), Index::default(), true, true))),
                ("-option=bar!/", Some((Some("-"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("-option=bar/!", Some((Some("-"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("--option=bar", Some((Some("--"), Some("option"), Some("bar"), Index::default(), false, true))),
                ("--option=bar!", Some((Some("--"), Some("option"), Some("bar"), Index::default(), false, false))),
                ("--option=bar/", Some((Some("--"), Some("option"), Some("bar"), Index::default(), true, true))),
                ("--option=bar!/", Some((Some("--"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("--option=bar/!", Some((Some("--"), Some("option"), Some("bar"), Index::default(), true, false))),
                ("=bar", Some((Some(""), None, Some("bar"), Index::default(), false, true))),
                ("=bar!", Some((Some(""), None, Some("bar"), Index::default(), false, false))),
                ("=bar/", Some((Some(""), None, Some("bar"), Index::default(), true, true))),
                ("=bar!/", Some((Some(""), None, Some("bar"), Index::default(), true, false))),
                ("=bar/!", Some((Some(""), None, Some("bar"), Index::default(), true, false))),
                ("option=bar@1", Some((Some(""), Some("option"), Some("bar"), Index::forward(1), false, true))),
                ("option=bar@-1", Some((Some(""), Some("option"), Some("bar"), Index::backward(1), false, true))),
                ("option=bar@+42", Some((Some(""), Some("option"), Some("bar"), Index::forward(42), false, true))),
                ("option=bar@[1, 2, 3]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, true))),
                ("option=bar@+[4, 5, 12]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, true))),
                ("option=bar@-[1, 2, 4]", Some((Some(""), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, true))),
                ("-option=bar@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), false, true))),
                ("-option=bar@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), false, true))),
                ("-option=bar@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), false, true))),
                ("-option=bar@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, true))),
                ("-option=bar@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, true))),
                ("-option=bar@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, true))),
                ("--option=bar@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), false, true))),
                ("--option=bar@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), false, true))),
                ("--option=bar@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), false, true))),
                ("--option=bar@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, true))),
                ("--option=bar@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, true))),
                ("--option=bar@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, true))),
                ("option=bar!@1", Some((Some(""), Some("option"), Some("bar"), Index::forward(1), false, false))),
                ("option=bar!@-1", Some((Some(""), Some("option"), Some("bar"), Index::backward(1), false, false))),
                ("option=bar!@+42", Some((Some(""), Some("option"), Some("bar"), Index::forward(42), false, false))),
                ("option=bar!@[1, 2, 3]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, false))),
                ("option=bar!@+[4, 5, 12]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, false))),
                ("option=bar!@-[1, 2, 4]", Some((Some(""), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, false))),
                ("-option=bar!@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), false, false))),
                ("-option=bar!@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), false, false))),
                ("-option=bar!@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), false, false))),
                ("-option=bar!@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, false))),
                ("-option=bar!@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, false))),
                ("-option=bar!@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, false))),
                ("--option=bar!@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), false, false))),
                ("--option=bar!@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), false, false))),
                ("--option=bar!@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), false, false))),
                ("--option=bar!@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), false, false))),
                ("--option=bar!@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), false, false))),
                ("--option=bar!@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), false, false))),
                ("option=bar/@1", Some((Some(""), Some("option"), Some("bar"), Index::forward(1), true, true))),
                ("option=bar/@-1", Some((Some(""), Some("option"), Some("bar"), Index::backward(1), true, true))),
                ("option=bar/@+42", Some((Some(""), Some("option"), Some("bar"), Index::forward(42), true, true))),
                ("option=bar/@[1, 2, 3]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, true))),
                ("option=bar/@+[4, 5, 12]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, true))),
                ("option=bar/@-[1, 2, 4]", Some((Some(""), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, true))),
                ("-option=bar/@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), true, true))),
                ("-option=bar/@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), true, true))),
                ("-option=bar/@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), true, true))),
                ("-option=bar/@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, true))),
                ("-option=bar/@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, true))),
                ("-option=bar/@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, true))),
                ("--option=bar/@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), true, true))),
                ("--option=bar/@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), true, true))),
                ("--option=bar/@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), true, true))),
                ("--option=bar/@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, true))),
                ("--option=bar/@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, true))),
                ("--option=bar/@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, true))),
                ("option=bar!/@1", Some((Some(""), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("option=bar!/@-1", Some((Some(""), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("option=bar!/@+42", Some((Some(""), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("option=bar!/@[1, 2, 3]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("option=bar!/@+[4, 5, 12]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("option=bar!/@-[1, 2, 4]", Some((Some(""), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("-option=bar!/@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("-option=bar!/@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("-option=bar!/@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("-option=bar!/@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("-option=bar!/@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("-option=bar!/@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("--option=bar!/@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("--option=bar!/@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("--option=bar!/@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("--option=bar!/@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("--option=bar!/@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("--option=bar!/@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("option=bar/!@1", Some((Some(""), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("option=bar/!@-1", Some((Some(""), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("option=bar/!@+42", Some((Some(""), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("option=bar/!@[1, 2, 3]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("option=bar/!@+[4, 5, 12]", Some((Some(""), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("option=bar/!@-[1, 2, 4]", Some((Some(""), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("-option=bar/!@1", Some((Some("-"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("-option=bar/!@-1", Some((Some("-"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("-option=bar/!@+42", Some((Some("-"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("-option=bar/!@[1, 2, 3]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("-option=bar/!@+[4, 5, 12]", Some((Some("-"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("-option=bar/!@-[1, 2, 4]", Some((Some("-"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
                ("--option=bar/!@1", Some((Some("--"), Some("option"), Some("bar"), Index::forward(1), true, false))),
                ("--option=bar/!@-1", Some((Some("--"), Some("option"), Some("bar"), Index::backward(1), true, false))),
                ("--option=bar/!@+42", Some((Some("--"), Some("option"), Some("bar"), Index::forward(42), true, false))),
                ("--option=bar/!@[1, 2, 3]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![1, 2, 3]), true, false))),
                ("--option=bar/!@+[4, 5, 12]", Some((Some("--"), Some("option"), Some("bar"), Index::list(vec![4, 5, 12]), true, false))),
                ("--option=bar/!@-[1, 2, 4]", Some((Some("--"), Some("option"), Some("bar"), Index::except(vec![1, 2, 4]), true, false))),
            ];

            let prefixs = vec![
                String::from("--"),
                String::from("-"),
                String::from(""),
            ];

            for case in test_cases.iter() {
                try_to_verify_one_task(case.0, &prefixs,&case.1);
            }
        }
    }

    fn try_to_verify_one_task(pattern: &str, prefix: &Vec<String>, except: &Option<(Option<&str>, Option<&str>, Option<&str>, Index, bool, bool)>) {
        let ret = parse_option_str(pattern, prefix);

        if let Ok(mut dk) = ret {
            assert!(except.is_some());

            let default = String::from("");

            if let Some(except) = except {
                let index = dk.gen_index();

                assert_eq!(except.0.unwrap_or(""), dk.prefix.unwrap_or(&default));
                assert_eq!(except.1.unwrap_or(""), dk.name.unwrap_or(default.clone()));
                assert_eq!(except.2.unwrap_or(""), dk.type_name.unwrap_or(default.clone()));
                assert_eq!(except.3, index);
                assert_eq!(except.4, dk.deactivate);
                assert_eq!(!except.5, dk.optional);
            }
        }
        else {
            assert!(except.is_none());
        }
    }
}