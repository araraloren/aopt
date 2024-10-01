use std::ffi::OsStr;

use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote(help, flag)]
pub struct Cli {
    flag: Option<Flag>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Flag;

impl Infer for Flag {
    type Val = Flag;

    fn infer_style() -> Vec<aopt::prelude::Style> {
        vec![Style::Flag]
    }
}

cote::impl_alter!(Flag);

cote::impl_fetch!(Flag);

impl RawValParser for Flag {
    type Error = cote::Error;

    fn parse(raw: Option<&OsStr>, _: &Ctx) -> Result<Self, Self::Error> {
        assert!(raw.is_none());
        Ok(Flag {})
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse(Args::from(["app", "--flag"].into_iter()))?;

    assert_eq!(cli.flag, Some(Flag {}));
    Ok(())
}
