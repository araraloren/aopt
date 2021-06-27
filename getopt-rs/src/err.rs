
use std::{ops::{Range, RangeFrom}, usize};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("invalid option string: `{0}`")]
	InvalidOptionStr(String),

	// for parse argument
	#[error("parse the option string failed: `{0}`")]
	InvalidArgAsOption(String),

	#[error("can not get string with range: {:?} .. {:?}", beg, end)]
	InvalidStrRange{
		beg: usize,
		end: usize,
	},

	#[error("option string with '=' need an value after it: `{0}`")]
	InvalidArgArgument(String),

	#[error("invalid option index value: `{0}`")]
	InavlidOptionIndexValue(String),
}



