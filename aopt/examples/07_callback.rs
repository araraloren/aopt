use aopt::prelude::*;

pub fn changed(other: &str) -> impl Fn(&mut AHCSet, &mut Ctx) -> aopt::Result<Option<bool>> + '_ {
    move |set, ctx| {
        let val = ctx.value::<bool>()?;
        *set.find_val_mut::<bool>(other)? = !val;
        Ok(Some(val))
    }
}

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    parser.add_opt("-flag;--flag=b")?.on(changed("--/flag"))?;
    parser
        .add_opt("-/flag;--/flag=b")?
        .set_value_t(true)
        .on(changed("--flag"))?;
    parser
        .parse(Args::from(["app", "-flag"].into_iter()))?
        .ok()?;

    assert_eq!(parser.find_val::<bool>("-flag")?, &true);
    assert_eq!(parser.find_val::<bool>("--/flag")?, &false);

    parser
        .parse(Args::from(["app", "--/flag"].into_iter()))?
        .ok()?;

    assert_eq!(parser.find_val::<bool>("-flag")?, &false);
    assert_eq!(parser.find_val::<bool>("--/flag")?, &true);

    Ok(())
}
