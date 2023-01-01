use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::Error;
use crate::RawVal;

pub trait RawValParser<Opt>
where
    Self: Sized,
{
    type Error: Into<Error>;

    fn parse(opt: &Opt, val: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error>;
}

macro_rules! impl_raw_val_parser {
    ($int:ty) => {
        impl<Opt: crate::opt::Opt> RawValParser<Opt> for $int {
            type Error = Error;

            fn parse(opt: &Opt, val: Option<&RawVal>, _ctx: &Ctx) -> Result<$int, Self::Error> {
                let name = opt.name().as_str();

                val.ok_or_else(|| Error::sp_missing_argument(name))?
                    .get_str()
                    .ok_or_else(|| {
                        Error::sp_invalid_option_value(
                            name,
                            &format!("Can't convert value to {}: invalid utf8", stringify!($int)),
                        )
                    })?
                    .parse::<$int>()
                    .map_err(|e| Error::sp_invalid_option_value(name.to_string(), e.to_string()))
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

impl<Opt: crate::opt::Opt> RawValParser<Opt> for String {
    type Error = Error;

    fn parse(opt: &Opt, val: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        let name = opt.name().as_str();

        val.ok_or_else(|| Error::sp_missing_argument(name))?
            .get_str()
            .map(|v| v.to_string())
            .ok_or_else(|| {
                Error::sp_invalid_option_value(name, "Can't convert value to String: invalid utf8")
            })
    }
}

impl<Opt: crate::opt::Opt> RawValParser<Opt> for bool {
    type Error = Error;

    fn parse(opt: &Opt, val: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        let name = opt.name().as_str();
        let val = val
            .ok_or_else(|| Error::sp_missing_argument(name))?
            .get_str()
            .ok_or_else(|| {
                Error::sp_invalid_option_value(name, "Can't convert value to bool: invalid utf8")
            })?;

        match val {
            crate::opt::BOOL_TRUE => Ok(true),
            crate::opt::BOOL_FALSE => Ok(false),
            _ => Err(Error::sp_invalid_option_value(
                name,
                &format!("Except true or false, found value: {}", val),
            )),
        }
    }
}

impl<Opt: crate::opt::Opt> RawValParser<Opt> for PathBuf {
    type Error = Error;

    fn parse(opt: &Opt, val: Option<&RawVal>, _ctx: &Ctx) -> Result<Self, Self::Error> {
        let name = opt.name().as_str();

        Ok(PathBuf::from(
            val.ok_or_else(|| Error::sp_missing_argument(name))?
                .get_str()
                .map(|v| v.to_string())
                .ok_or_else(|| {
                    Error::sp_invalid_option_value(
                        name,
                        "Can't convert value to String: invalid utf8",
                    )
                })?,
        ))
    }
}
