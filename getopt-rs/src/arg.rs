

pub mod argument;
pub mod parser;

use std::fmt::Debug;
use std::iter::ExactSizeIterator;
use std::marker::PhantomData;
use std::usize;

use crate::str::Str;
use crate::err::{Error, Result};

use argument::Argument;

pub trait ArgIterator<'str, 'nv, 'pre>: Debug + ExactSizeIterator<Item = Argument<'str, 'nv, 'pre>> { }

#[derive(Debug)]
pub struct ArgStream<'arg, 'nv, 'pre> {
    index: usize,

    args: Vec<Str<'arg>>,

    prefixs: Vec<Str<'pre>>,

    phantom: Option<Str<'nv>>,
}

impl<'arg, 'nv, 'pre> ArgIterator<'arg, 'nv, 'pre> for ArgStream<'arg, 'nv, 'pre> { }

impl<'arg, 'nv, 'pre> ExactSizeIterator for ArgStream<'arg, 'nv, 'pre> {
    fn len(&self) -> usize {
        self.args.len()
    }
}

impl<'arg, 'nv, 'pre> Iterator for ArgStream<'arg, 'nv, 'pre> {
    type Item = Argument<'arg, 'nv, 'pre>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.index;

        if current < self.len() {
            return Some(Argument::new(
                Some(self.args[current].clone()),
                if current < self.len() - 1 {
                    Some(self.args[current + 1].clone())
                }
                else {
                    None
                }
            ))
        }
        None
    }
}
