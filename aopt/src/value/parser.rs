use std::ffi::OsString;
use std::io::Stdin;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::value::Stop;
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

fn ok_or_else(raw: Option<&RawVal>) -> Result<&RawVal, Error> {
    raw.ok_or_else(|| Error::sp_rawval(None, "unexcepted empty value"))
}

/// Convert raw value to &[`str`].
pub fn raw2str(raw: Option<&RawVal>) -> Result<&str, Error> {
    ok_or_else(raw)?
        .get_str()
        .ok_or_else(|| Error::sp_rawval(raw, "can not convert RawVal to str"))
}

impl RawValParser for () {
    type Error = Error;

    fn parse(_: Option<&RawVal>, _: &Ctx) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl RawValParser for RawVal {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _: &Ctx) -> Result<Self, Self::Error> {
        ok_or_else(raw).cloned()
    }
}

macro_rules! impl_raw_val_parser {
    ($int:ty) => {
        impl RawValParser for $int {
            type Error = Error;

            fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<$int, Self::Error> {
                let val = $crate::value::parser::raw2str(raw)?;
                let uid = ctx.uid()?;

                val.parse::<$int>().map_err(|e| {
                    $crate::err::Error::sp_rawval(
                        raw.clone(),
                        format!("not a valid value of type {}", stringify!($int)),
                    )
                    .with_uid(uid)
                    .cause_by(e.into())
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
        Ok(raw2str(raw)?.to_string())
    }
}

impl RawValParser for OsString {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let uid = ctx.uid()?;

        Ok(ok_or_else(raw).map_err(|e| e.with_uid(uid))?.to_os_string())
    }
}

impl RawValParser for bool {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let val = raw2str(raw)?;

        match val {
            crate::opt::BOOL_TRUE => Ok(true),
            crate::opt::BOOL_FALSE => Ok(false),
            _ => Err(Error::sp_rawval(raw, "except true or false").with_uid(ctx.uid()?)),
        }
    }
}

impl RawValParser for PathBuf {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(PathBuf::from(ok_or_else(raw)?.clone().into_os_string()))
    }
}

impl RawValParser for Stdin {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        const STDIN: &str = "-";

        if ctx.name()?.map(|v| v.as_str()) == Some(STDIN) {
            Ok(std::io::stdin())
        } else {
            Err(Error::sp_rawval(raw, "except `-` for Stdin").with_uid(ctx.uid()?))
        }
    }
}

impl RawValParser for Stop {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        const STOP: &str = "--";

        if ctx.name()?.map(|v| v.as_str()) == Some(STOP) {
            ctx.set_policy_act(crate::parser::Action::StopPolicy);
            Ok(Stop)
        } else {
            Err(Error::sp_rawval(raw, "except `--` for Stop").with_uid(ctx.uid()?))
        }
    }
}
