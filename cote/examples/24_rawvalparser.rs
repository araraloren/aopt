use cote::*;

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
#[fetch(inner = i32, map = Speed)]
#[infer(val = i32)]
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
}

impl Fetch<'_> for Direction {}

impl Alter for Direction {}

impl cote::RawValParser for Direction {
    type Error = cote::aopt::Error;

    fn parse(raw: Option<&cote::RawVal>, ctx: &cote::Ctx) -> Result<Self, Self::Error> {
        let name = cote::raw2str(raw)?.to_lowercase();
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

    let cli = Cli::parse(Args::from_array(["app", "-s", "40", "-d=Left", "bike"]))?;

    assert_eq!(cli.speed.0, 40);
    assert_eq!(cli.direction, Direction::Left);
    assert_eq!(cli.way, Way::Bike);

    Ok(())
}
