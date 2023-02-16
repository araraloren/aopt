use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Stdin;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::Error;
use crate::RawVal;

/// Implement this if you want parsing the raw value into your type.
pub trait RawValParser
where
    Self: Sized,
{
    type Error: Into<Error>;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error>;
}

/// Convert raw value to &[`str`].
pub fn raw2str(raw: Option<&RawVal>) -> Result<&str, Error> {
    raw.ok_or_else(|| Error::raise_failure("unexcepted empty value"))?
        .get_str()
        .ok_or_else(|| {
            Error::raise_failure(format!(
                "Can't convert value `{:?}` to &str: invalid utf8",
                raw
            ))
        })
}

impl RawValParser for () {
    type Error = Error;

    fn parse(_: Option<&RawVal>, _: &Ctx) -> Result<Self, Self::Error> {
        Ok(())
    }
}

macro_rules! impl_raw_val_parser {
    ($int:ty) => {
        impl RawValParser for $int {
            type Error = Error;

            fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<$int, Self::Error> {
                let val = $crate::value::parser::raw2str(raw)?;

                val.parse::<$int>().map_err(|e| {
                    Error::raise_failure(format!(
                        "Can not convert value `{:?}` to {}: {:?}",
                        raw,
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

#[cfg(not(feature = "utf8"))]
impl RawValParser for String {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(raw2str(raw)?.to_string())
    }
}

#[cfg(not(feature = "utf8"))]
impl RawValParser for OsString {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Self::clone(raw.ok_or_else(|| {
            Error::raise_failure("unexcepted empty value")
        })?))
    }
}

#[cfg(feature = "utf8")]
impl RawValParser for String {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(raw2str(raw)?.to_owned())
    }
}

#[cfg(feature = "utf8")]
impl RawValParser for OsString {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        let raw: &OsStr = raw
            .ok_or_else(|| Error::raise_failure("unexcepted empty value"))?
            .as_ref();
        Ok(raw.to_owned())
    }
}

impl RawValParser for bool {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        let val = raw2str(raw)?;

        match val {
            crate::opt::BOOL_TRUE => Ok(true),
            crate::opt::BOOL_FALSE => Ok(false),
            _ => Err(Error::raise_failure(format!(
                "Except true or false, found value: {}",
                val
            ))),
        }
    }
}

impl RawValParser for PathBuf {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(PathBuf::from(raw2str(raw)?))
    }
}

impl RawValParser for Stdin {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _: &Ctx) -> Result<Self, Self::Error> {
        const STDIN: &str = "-";

        if let Some(raw) = raw {
            if raw.get_str() == Some(STDIN) {
                return Ok(std::io::stdin());
            }
        }
        Err(Error::raise_failure(format!(
            "Stdin value only support value `-`: {:?}",
            raw
        )))
    }
}
