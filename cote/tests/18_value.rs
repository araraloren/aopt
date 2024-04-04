use aopt::prelude::AFwdParser;
use cote::prelude::*;

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

#[test]
fn value() {
    assert!(value_impl().is_ok());
}

fn value_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut parser = AFwdParser::default();

    parser.add_opt("--speed".infer::<Speed>())?;
    parser.add_opt("-im;--instant-message".infer::<IM>())?;
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
