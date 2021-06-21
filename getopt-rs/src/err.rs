
use std::{ops::{Range, RangeFrom}, usize};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("invalid option string: `{0}`")]
	InvalidOptionStr(String),

	#[error("can not get string with range: {:?} .. {:?}", beg, end)]
	InvalidStrRange{
		beg: usize,
		end: usize,
	},
}



