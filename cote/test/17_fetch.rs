use cote::{aopt::prelude::AFwdParser, *};

#[derive(Debug, PartialEq, Eq, CoteOpt)]
#[infer(val = i32)]
#[fetch(inner = i32, map = Speed)]
pub struct Speed(i32);

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut parser = AFwdParser::default();

    parser.add_opt_i::<i32>("--speed")?;
    parser.parse(ARef::new(Args::from(["app", "--speed=42"].into_iter())))?;
    
    assert_eq!(Speed::fetch("--speed", parser.optset_mut())?, Speed(42));

    Ok(())
}
