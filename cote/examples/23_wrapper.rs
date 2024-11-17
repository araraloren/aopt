use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Cli {
    #[arg(alias = "-s")]
    speed: Speed,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Speed(i32);

impl Infer for Speed {
    type Val = i32;
}

impl<'a, S> Fetch<'a, S> for Speed
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
    Self: ErasedTy + Sized,
{
    fn fetch_uid(uid: Uid, set: &'a mut S) -> cote::Result<Self> {
        Ok(Speed(fetch_uid_impl(uid, set)?))
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from(["app", "--speed", "65"]))?;

    assert_eq!(cli.speed.0, 65);

    Ok(())
}
