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

impl<Opt: crate::opt::Opt> RawValParser<Opt> for i32 {
    type Error = Error;

    fn parse(opt: &Opt, val: Option<&RawVal>, _ctx: &Ctx) -> Result<i32, Self::Error> {
        let name = opt.name().as_str();

        val.ok_or_else(|| Error::sp_missing_argument(name))?
            .to_str()
            .ok_or_else(|| {
                Error::sp_invalid_option_value(name, "Can't convert value to i64: invalid utf8")
            })?
            .parse::<i32>()
            .map_err(|e| Error::sp_invalid_option_value(name.to_string(), e.to_string()))
    }
}

impl<Opt: crate::opt::Opt> RawValParser<Opt> for i64 {
    type Error = Error;

    fn parse(opt: &Opt, val: Option<&RawVal>, _ctx: &Ctx) -> Result<i64, Self::Error> {
        let name = opt.name().as_str();

        val.ok_or_else(|| Error::sp_missing_argument(name))?
            .to_str()
            .ok_or_else(|| {
                Error::sp_invalid_option_value(name, "Can't convert value to i64: invalid utf8")
            })?
            .parse::<i64>()
            .map_err(|e| Error::sp_invalid_option_value(name.to_string(), e.to_string()))
    }
}
