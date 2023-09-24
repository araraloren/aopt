use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    // add option need argument with type `i`, i.e. i64
    parser.add_opt("-f=i")?;
    parser.add_opt("--flag=s")?;
    parser.add_opt_i::<f32>("-flag")?;

    parser.init()?;
    parser
        .parse(ARef::new(Args::from(
            ["app", "-f42", "--flag", "foo", "-flag=2.1"].into_iter(),
        )))?
        .ok()?;

    assert_eq!(parser.find_val::<i64>("-f")?, &42);
    assert_eq!(parser.find_val::<String>("--flag")?, "foo");
    assert_eq!(parser.find_val::<f32>("-flag")?, &2.1);

    Ok(())
}
