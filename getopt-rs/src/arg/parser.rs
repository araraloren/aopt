
use crate::str::Str;
use crate::err::{Result, Error};
use crate::pattern::ParserPattern;

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
pub struct DataKeeper<'a, 'b> {
    pub name: Option<Str<'a>>,

    pub value: Option<Str<'a>>,

    pub prefix: Option<Str<'b>>,

    pub disable: bool,
}

impl Default for State {
    fn default() -> Self {
        Self::PreCheck
    }
}

impl State {
    pub fn self_transition<'a, 'b, 'c>(&mut self, pattern: ParserPattern<'a, 'b, 'c>) {

    }
}