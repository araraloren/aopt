use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::Error;
use crate::RawVal;

pub trait RawValParser
where
    Self: Sized,
{
    type Error: Into<Error>;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error>;
}

pub(crate) fn convert_raw_to_utf8(raw: Option<&RawVal>) -> Result<&str, Error> {
    raw.ok_or_else(|| Error::raise_failure("unexcepted empty value"))?
        .get_str()
        .ok_or_else(|| Error::raise_failure("Can't convert value to &str: invalid utf8"))
}

macro_rules! impl_raw_val_parser {
    ($int:ty) => {
        impl RawValParser for $int {
            type Error = Error;

            fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<$int, Self::Error> {
                let val = convert_raw_to_utf8(raw)?;

                val.parse::<$int>().map_err(|e| {
                    Error::raise_failure(format!(
                        "Can not convert value to {}: {:?}",
                        stringify!($int),
                        e
                    ))
                })
            }
        }
    };
}

impl_raw_val_parser!(i8);
impl_raw_val_parser!(i16);
impl_raw_val_parser!(i32);
impl_raw_val_parser!(i64);
impl_raw_val_parser!(i128);
impl_raw_val_parser!(u8);
impl_raw_val_parser!(u16);
impl_raw_val_parser!(u32);
impl_raw_val_parser!(u64);
impl_raw_val_parser!(u128);
impl_raw_val_parser!(f32);
impl_raw_val_parser!(f64);
impl_raw_val_parser!(isize);
impl_raw_val_parser!(usize);

impl RawValParser for String {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(convert_raw_to_utf8(raw)?.to_string())
    }
}

impl RawValParser for bool {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        let val = convert_raw_to_utf8(raw)?;

        match val {
            crate::opt::BOOL_TRUE => Ok(true),
            crate::opt::BOOL_FALSE => Ok(false),
            _ => Err(Error::raise_failure(&format!(
                "Except true or false, found value: {}",
                val
            ))),
        }
    }
}

impl RawValParser for PathBuf {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(PathBuf::from(convert_raw_to_utf8(raw)?))
    }
}
