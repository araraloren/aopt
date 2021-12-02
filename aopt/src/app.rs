use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use crate::arg::ArgStream;
use crate::err::Result;
use crate::opt::OptCallback;
use crate::parser::HashMapIter;
use crate::parser::Parser;
use crate::set::Set;
use crate::uid::Uid;

#[derive(Debug, Default)]
pub struct SingleApp<S: Set + Default, P: Parser + Default> {
    name: String,
    set: S,
    parser: P,
}

impl<S: Set + Default, P: Parser + Default> SingleApp<S, P> {
    pub fn new(name: String, set: S, parser: P) -> Self {
        Self { name, set, parser }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_optset(mut self, set: S) -> Self {
        self.set = set;
        self
    }

    pub fn with_parser(mut self, parser: P) -> Self {
        self.parser = parser;
        self
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_optset(&self) -> &S {
        &self.set
    }

    pub fn get_parser(&mut self) -> &P {
        &self.parser
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_optset(&mut self, set: S) {
        self.set = set;
    }

    pub fn set_parser(&mut self, parser: P) {
        self.parser = parser;
    }

    pub fn add_callback(&mut self, uid: Uid, callback: OptCallback) {
        self.parser.add_callback(uid, callback);
    }

    pub fn get_callback(&self, uid: Uid) -> Option<&RefCell<OptCallback>> {
        self.parser.get_callback(uid)
    }

    pub fn callback_iter(&self) -> HashMapIter<'_, Uid, RefCell<OptCallback>> {
        self.parser.callback_iter()
    }

    pub fn run<RET, F: FnMut(bool, SingleApp<S, P>) -> Result<RET>>(
        &mut self,
        iter: impl Iterator<Item = String>,
        mut r: F,
    ) -> Result<RET> {
        let set = &mut self.set;
        let parser = &mut self.parser;
        let ret = parser.parse(set, &mut ArgStream::from(iter))?;
        let _self = std::mem::take(self);

        r(ret, _self)
    }
}

// Implement Deref/DerefMut for SingleApp.
impl<S: Set + Default, P: Parser + Default> Deref for SingleApp<S, P> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<S: Set + Default, P: Parser + Default> DerefMut for SingleApp<S, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}
