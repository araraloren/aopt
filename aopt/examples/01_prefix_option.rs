use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    // add option with value type `b`, i.e. option with bool value or flag
    parser.add_opt("-f=b")?;
    parser.add_opt("--flag".infer::<bool>())?;

    parser.parse(Args::from(["app", "-f"].into_iter()))?.ok()?;

    // option with bool type has default value `false`
    assert_eq!(parser.find_val::<bool>("-f")?, &true);
    assert_eq!(parser.find_val::<bool>("--flag")?, &false);

    parser.add_opt("-flag=b!")?;
    parser.add_opt("--/flag=b")?;

    parser
        .parse(Args::from(["app", "-flag", "--/flag"].into_iter()))?
        .ok()?;

    // option with bool type has default value `false`
    assert_eq!(parser.find_val::<bool>("-flag")?, &true);
    assert_eq!(parser.find_val::<bool>("--/flag")?, &true);

    Ok(())
}
