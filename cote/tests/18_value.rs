use cote::{aopt::prelude::AFwdParser, *};

#[derive(Debug, PartialEq, Eq, CoteOpt, CoteVal)]
#[coteval(forward = i32, map = Speed)]
pub struct Speed(i32);

#[derive(Debug, CoteVal, CoteOpt, PartialEq)]
#[coteval(igcase)]
pub enum IM {
    #[coteval(alias = "qq")]
    OICQ,

    ICQ,

    Line,

    Skype,

    WeChat,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut parser = AFwdParser::default();

    parser.add_opt_i::<Speed>("--speed")?;
    parser.add_opt_i::<IM>("-im;--instant-message")?;
    parser.parse(ARef::new(Args::from(
        ["app", "--speed=42", "-im=qq"].into_iter(),
    )))?;

    assert_eq!(Speed::fetch("--speed", parser.optset_mut())?, Speed(42));
    assert_eq!(
        IM::fetch("--instant-message", parser.optset_mut())?,
        IM::OICQ
    );
    Ok(())
}
