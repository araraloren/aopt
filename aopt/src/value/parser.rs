use std::ffi::OsString;
use std::io::Stdin;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::raise_command;
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

/// Convert raw value to &[`str`].
pub fn raw2str(raw: Option<&RawVal>) -> Result<&str, Error> {
    let raw = raw.ok_or_else(|| Error::raise_sp_rawval("Unexcepted empty value"))?;

    raw.get_str()
        .ok_or_else(|| Error::raise_sp_rawval(format!("Can't convert value `{raw}` to str")))
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
        raw.cloned()
            .ok_or_else(|| Error::raise_sp_rawval("Unexcepted empty value"))
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
                    $crate::err::Error::raise_sp_rawval(format!(
                        "Can not convert value `{val}` to {}",
                        stringify!($int)
                    ))
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
        Ok(raw
            .ok_or_else(|| Error::raise_sp_rawval("Unexcepted empty value").with_uid(uid))?
            .to_os_string())
    }
}

impl RawValParser for bool {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let val = raw2str(raw)?;

        match val {
            crate::opt::BOOL_TRUE => Ok(true),
            crate::opt::BOOL_FALSE => Ok(false),
            _ => Err(
                Error::raise_sp_rawval(format!("Except true or false, found value: {}", val))
                    .with_uid(ctx.uid()?),
            ),
        }
    }
}

impl RawValParser for PathBuf {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(PathBuf::from(
            raw.ok_or_else(|| Error::raise_sp_rawval("Can not construct PathBuf from None"))?
                .clone()
                .into_os_string(),
        ))
    }
}

impl RawValParser for Stdin {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        const STDIN: &str = "-";

        if let Some(raw) = raw {
            if raw.get_str() == Some(STDIN) {
                return Ok(std::io::stdin());
            }
        }
        Err(
            Error::raise_sp_rawval(format!("Stdin value only support value `-`: {raw:?}"))
                .with_uid(ctx.uid()?),
        )
    }
}

impl RawValParser for Stop {
    type Error = Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        const STOP: &str = "--";

        let inner_ctx = ctx.inner_ctx()?;

        match inner_ctx.style() {
            crate::prelude::Style::Null => {
                unreachable!("Unexcepted null style in ctx({:?})", ctx)
            }
            crate::prelude::Style::Pos
            | crate::prelude::Style::Cmd
            | crate::prelude::Style::Main => {
                // check value for noa
                if let Some(raw) = raw {
                    if raw.get_str() == Some(STOP) {
                        return Err(raise_command!(crate::err::ErrorCmd::StopPolicy));
                    }
                }
            }
            crate::prelude::Style::Boolean
            | crate::prelude::Style::Argument
            | crate::prelude::Style::Combined
            | crate::prelude::Style::Flag => {
                // check name for option
                if inner_ctx.name().map(|v| v.as_str()) == Some(STOP) {
                    return Err(raise_command!(crate::err::ErrorCmd::StopPolicy));
                }
            }
        }
        Err(
            Error::raise_sp_rawval(format!("Stop value only support value `--`: {raw:?}"))
                .with_uid(ctx.uid()?),
        )
    }
}
