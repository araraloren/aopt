use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    parser.add_opt("-flag".infer::<i32>())?;
    parser.add_opt("-flag".infer::<String>())?;
    parser.add_opt("-flag".infer::<bool>())?;

    // enable combination style
    parser.set_overload(true);
    parser
        .parse(Args::from(
            ["app", "-flag=foo", "-flag=42", "-flag"].into_iter(),
        ))?
        .ok()?;

    assert_eq!(parser.find("-flag".infer::<i32>())?.val::<i32>()?, &42);
    assert_eq!(
        parser.find("-flag".infer::<String>())?.val::<String>()?,
        "foo"
    );
    assert_eq!(parser.find("-flag".infer::<bool>())?.val::<bool>()?, &true);

    Ok(())
}
