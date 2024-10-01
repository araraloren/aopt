use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Stdin;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::value::Stop;
use crate::Error;

/// Implement this if you want parsing the raw value into your type.
pub trait RawValParser
where
    Self: Sized,
{
    type Error: Into<Error>;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<Self, Self::Error>;
}

fn ok_or_else(raw: Option<&OsStr>) -> Result<&OsStr, Error> {
    raw.ok_or_else(|| Error::sp_rawval(None, "unexcepted empty value"))
}

/// Convert raw value to &[`str`].
pub fn raw2str(raw: Option<&OsStr>) -> Result<&str, Error> {
    ok_or_else(raw)?
        .to_str()
        .ok_or_else(|| Error::sp_rawval(raw, "can not convert RawVal to str"))
}

impl RawValParser for () {
    type Error = Error;

    fn parse(_: Option<&OsStr>, _: &Ctx) -> Result<Self, Self::Error> {
        Ok(())
    }
}

macro_rules! impl_raw_val_parser {
    ($int:ty) => {
        impl $crate::value::parser::RawValParser for $int {
            type Error = Error;

            fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<$int, Self::Error> {
                let val = $crate::value::parser::raw2str(raw)?;
                let uid = ctx.uid()?;

                val.parse::<$int>().map_err(|e| {
                    $crate::err::Error::sp_rawval(
                        raw,
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

    fn parse(raw: Option<&OsStr>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(raw2str(raw)?.to_string())
    }
}

impl RawValParser for OsString {
    type Error = Error;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let uid = ctx.uid()?;

        Ok(ok_or_else(raw).map_err(|e| e.with_uid(uid))?.to_os_string())
    }
}

impl RawValParser for bool {
    type Error = Error;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<Self, Self::Error> {
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

    fn parse(raw: Option<&OsStr>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(PathBuf::from(ok_or_else(raw)?))
    }
}

/// A special option value, using for implement `-`.
///
/// # Example
/// ```
/// use aopt::prelude::*;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let mut parser = AFwdParser::default();
///
///     parser.set_strict(true);
///     parser.add_opt("stdin=b".infer::<std::io::Stdin>())?;
///
///     // -w will processed, it is set before `--`
///     parser.add_opt("-w=i")?;
///
///     // -o will not processed, it is set after `--`
///     parser.add_opt("-o=s")?;
///
///     // fo will processed, it is not an option
///     parser.add_opt("foo=p@1")?;
///
///     parser.parse(Args::from(
///         ["app", "-w=42", "-", "foo"].into_iter(),
///     ))?;
///
///     assert_eq!(parser.find_val::<i64>("-w")?, &42);
///     assert!(parser.find_val::<std::io::Stdin>("-").is_ok());
///     assert_eq!(parser.find_val::<bool>("foo")?, &true);
///     Ok(())
/// }
/// ```
impl RawValParser for Stdin {
    type Error = Error;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<Self, Self::Error> {
        const STDIN: &str = "-";

        if ctx.name()?.map(|v| v.as_ref()) == Some(STDIN) {
            Ok(std::io::stdin())
        } else {
            Err(Error::sp_rawval(raw, "except `-` for Stdin").with_uid(ctx.uid()?))
        }
    }
}

impl RawValParser for Stop {
    type Error = Error;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<Self, Self::Error> {
        const STOP: &str = "--";

        if ctx.name()?.map(|v| v.as_ref()) == Some(STOP) {
            ctx.set_policy_act(crate::parser::Action::Stop);
            Ok(Stop)
        } else {
            Err(Error::sp_rawval(raw, "except `--` for Stop").with_uid(ctx.uid()?))
        }
    }
}
