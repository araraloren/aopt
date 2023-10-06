use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    parser.add_opt("--flag=b")?;
    parser.add_opt("--foo=s")?;
    // In default, positional has bool value
    parser.add_opt("first=p@1")?;
    // A special positional argument match the name, and force required
    parser.add_opt("list=c")?;
    // Add a positional argument has String value
    parser.add_opt_i::<Pos<String>>("second@2")?;

    // enable combination style
    parser.enable_combined();
    parser
        .parse(ARef::new(Args::from(
            ["app", "list", "--foo", "value", "bar"].into_iter(),
        )))?
        .ok()?;

    assert_eq!(parser.find_val::<bool>("list")?, &true);
    assert_eq!(parser.find_val::<bool>("first")?, &true);
    assert_eq!(parser.find_val::<String>("second")?, "bar");

    Ok(())
}
