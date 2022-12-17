use std::ops::{Deref, DerefMut};

use crate::RawVal;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReturnVal {
    status: bool,

    args: Vec<RawVal>,
}

impl ReturnVal {
    pub fn new(args: Vec<RawVal>, status: bool) -> Self {
        Self { status, args }
    }

    pub fn args(&self) -> &Vec<RawVal> {
        &self.args
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn take_args(&mut self) -> Vec<RawVal> {
        std::mem::take(&mut self.args)
    }

    pub fn into_args(mut self) -> Vec<RawVal> {
        self.take_args()
    }
}

impl Deref for ReturnVal {
    type Target = Vec<RawVal>;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

impl DerefMut for ReturnVal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.args
    }
}
