use aopt::prelude::*;

pub fn main() -> Result<(), aopt::Error> {
    let mut parser = AFwdParser::default();

    // combination style only support bool type
    parser.add_opt("-a=b")?;
    parser.add_opt("-b=b")?;
    parser.add_opt_i::<bool>("-c")?;
    parser.add_opt("d=b")?;
    parser.add_opt("e=b")?;
    parser.add_opt_i::<bool>("f")?;

    // enable combination style
    parser.enable_combined().init()?;
    parser
        .parse(ARef::new(Args::from(["app", "-abc", "def"].into_iter())))?
        .ok()?;

    assert_eq!(parser.find_val::<bool>("-a")?, &true);
    assert_eq!(parser.find_val::<bool>("-b")?, &true);
    assert_eq!(parser.find_val::<bool>("-c")?, &true);
    assert_eq!(parser.find_val::<bool>("d")?, &false);
    assert_eq!(parser.find_val::<bool>("e")?, &false);
    assert_eq!(parser.find_val::<bool>("f")?, &false);

    // for support non-prefix option, need add prefix `""`,
    // and disable the strict flag of policy
    parser.validator_mut().add_prefix("");
    parser.set_strict(false);
    parser.init()?;
    parser
        .parse(ARef::new(Args::from(["app", "-abc", "def"].into_iter())))?
        .ok()?;

    assert_eq!(parser.find_val::<bool>("-a")?, &true);
    assert_eq!(parser.find_val::<bool>("-b")?, &true);
    assert_eq!(parser.find_val::<bool>("-c")?, &true);
    assert_eq!(parser.find_val::<bool>("d")?, &true);
    assert_eq!(parser.find_val::<bool>("e")?, &true);
    assert_eq!(parser.find_val::<bool>("f")?, &true);

    Ok(())
}
