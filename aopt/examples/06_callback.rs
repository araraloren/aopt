use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    parser
        .add_opt_i::<i32>("-flag")?
        .on(|_: &mut ASet, _: &mut ASer| {
            println!("ignore the value set from command line");
            Ok(Some(42))
        })?;
    parser
        .add_opt_i::<bool>("--/flag")?
        .set_value(true)
        .on(|_: &mut ASet, _: &mut ASer, val: ctx::Value<bool>| Ok(Some(!*val)))?;

    parser.init()?;
    parser
        .parse(ARef::new(Args::from(
            ["app", "-flag=foo", "--/flag"].into_iter(),
        )))?
        .ok()?;

    assert_eq!(parser.find_val::<i32>("-flag")?, &42);
    assert_eq!(parser.find_val::<bool>("--/flag")?, &false);

    Ok(())
}
