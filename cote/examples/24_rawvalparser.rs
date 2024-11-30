use std::ffi::OsStr;

use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Cli {
    #[arg(alias = "-s")]
    speed: Speed,

    #[arg(alias = "-d")]
    direction: Direction,

    #[pos()]
    way: Way,
}

#[derive(Debug, PartialEq, Eq, CoteOpt)]
#[infer(val = i32, map = Speed)]
pub struct Speed(i32);

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Infer for Direction {
    type Val = Direction;

    fn infer_map(val: Self::Val) -> Self {
        val
    }
}

impl InferOverride for Direction {}

impl<S> Fetch<S> for Direction
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
    Self: ErasedTy + Sized,
{
}

impl RawValParser for Direction {
    type Error = cote::Error;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> cote::Result<Self> {
        let name = raw2str(raw)?.to_lowercase();
        let uid = ctx.uid()?;

        match name.as_str() {
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            "right" => Ok(Direction::Right),
            _ => Err(
                raise_failure!("Unknow value for enum type `{0}`: {1}", "Direction", name)
                    .with_uid(uid),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq, CoteVal, CoteOpt)]
#[coteval(igcase)]
pub enum Way {
    Walk,
    Bike,
    Roll,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from(["app", "-s", "40", "-d=Left", "bike"]))?;

    assert_eq!(cli.speed.0, 40);
    assert_eq!(cli.direction, Direction::Left);
    assert_eq!(cli.way, Way::Bike);

    Ok(())
}
