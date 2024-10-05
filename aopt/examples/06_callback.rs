use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    parser
        .add_opt("-flag".infer::<i32>())?
        .on(|_: &mut ASet, _: &mut ASer, _: &Ctx| {
            println!("ignore the value set from command line");
            Ok(Some(42))
        })?;
    parser
        .add_opt("--/flag".infer::<bool>())?
        .set_value(true)
        .on(|_: &mut ASet, _: &mut ASer, ctx: &Ctx| Ok(Some(!ctx.value::<bool>()?)))?;

    parser
        .parse(Args::from(["app", "-flag=foo", "--/flag"].into_iter()))?
        .ok()?;

    assert_eq!(parser.find_val::<i32>("-flag")?, &42);
    assert_eq!(parser.find_val::<bool>("--/flag")?, &false);

    Ok(())
}
