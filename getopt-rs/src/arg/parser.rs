
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
        let mut next_state = Self::End;

        match self.clone() {
            Self::PreCheck => {
                next_state = Self::Prefix;
            }
            Self::Prefix => {
                if let Some(ch) = pattern.left_chars().nth(0) {
                    // match the deactivate char
                    next_state = if ch == '/' { Self::Disable } else { Self::Name };
                }
            }
            Self::Disable => {
                next_state = Self::Name;
            }
            Self::Name => {
                if let Some(ch) = pattern.left_chars().nth(0) {
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

    pub fn parse(
        &mut self,
        pattern: &mut ParserPattern<'a, 'b, 'c>
    ) {
        
    }
}