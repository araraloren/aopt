use cote::*;

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

impl<'a> InferValueMut<'a> for Speed {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: Sized,
    {
        Ok(Speed(set.take_val(name)?))
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from_array(["app", "--speed", "65"]))?;

    assert_eq!(cli.speed.0, 65);

    Ok(())
}
