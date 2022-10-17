use crate::ctx::Ctx;
use crate::Error;
use crate::RawVal;

pub trait RawValParser<T> {
    fn parse(raw_val: Option<RawVal>, ctx: &Ctx) -> Result<T, Error>;
}
