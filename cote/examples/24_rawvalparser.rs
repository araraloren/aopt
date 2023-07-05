use cote::{aopt::value::raw2str, *};

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

#[derive(Debug, PartialEq, Eq)]
pub struct Speed(i32);

impl Infer for Speed {
    type Val = i32;
}

impl<'a> ValueFetch<'a> for Speed {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: Sized,
    {
        Ok(Speed(set.take_val(name)?))
    }
}

impl_alter!(Speed);

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Infer for Direction {
    type Val = Direction;
}

impl<'a> ValueFetch<'a> for Direction {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: Sized,
    {
        Ok(set.take_val(name)?)
    }
}

impl_alter!(Direction);

impl RawValParser for Direction {
    type Error = aopt::Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let name = raw2str(raw)?.to_lowercase();
        let uid = ctx.uid()?;

        match name.as_str() {
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            "right" => Ok(Direction::Right),
            _ => Err(aopt::raise_failure!("Unknow value for Direction: {}", name).with_uid(uid)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Way {
    Walk,
    Bike,
    Roll,
}

impl Infer for Way {
    type Val = Way;
}

cote::impl_value_fetch!(Way);

impl RawValParser for Way {
    type Error = aopt::Error;

    fn parse(raw: Option<&RawVal>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let name = raw2str(raw)?.to_lowercase();
        let uid = ctx.uid()?;

        match name.as_str() {
            "walk" => Ok(Way::Walk),
            "bike" => Ok(Way::Bike),
            "roll" => Ok(Way::Roll),
            _ => Err(aopt::raise_failure!("Unknow value for Way: {}", name).with_uid(uid)),
        }
    }
}

impl_alter!(Way);

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from_array(["app", "-s", "40", "-d=Left", "bike"]))?;

    assert_eq!(cli.speed.0, 40);
    assert_eq!(cli.direction, Direction::Left);
    assert_eq!(cli.way, Way::Bike);

    Ok(())
}
