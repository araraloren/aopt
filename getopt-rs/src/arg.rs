

pub mod argument;
pub mod parser;

use std::iter::ExactSizeIterator;

use crate::str::Str;
use crate::err::{Error, Result};

use argument::Argument;

pub trait ArgIterator<'a, 'b>: ExactSizeIterator<Item = Argument<'a, 'b>> { }

