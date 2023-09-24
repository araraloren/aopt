use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    parser.add_opt_i::<i32>("-flag")?;
    parser.add_opt_i::<String>("-flag")?;
    parser.add_opt_i::<bool>("-flag")?;

    // enable combination style
    parser.set_overload(true).init()?;
    parser
        .parse(ARef::new(Args::from(
            ["app", "-flag=foo", "-flag=42", "-flag"].into_iter(),
        )))?
        .ok()?;

    assert_eq!(parser.find_i::<i32>("-flag")?.unwrap().val::<i32>()?, &42);
    assert_eq!(
        parser.find_i::<String>("-flag")?.unwrap().val::<String>()?,
        "foo"
    );
    assert_eq!(
        parser.find_i::<bool>("-flag")?.unwrap().val::<bool>()?,
        &true
    );

    Ok(())
}
