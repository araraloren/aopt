use std::ops::Add;

use cote::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    #[arg(on = plus_one, then = plus_two)]
    value: i64,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse(Args::from(["app", "--value=39"].into_iter()))?;
    assert_eq!(cli.value, 42);
    Ok(())
}

fn plus_one<Set, Ser>(
    _: &mut Set,
    _: &mut Ser,
    val: ctx::Value<i64>, // extract value from argument
) -> Result<Option<i64>, CoteError> {
    Ok(Some(val.add(1)))
}

fn plus_two<Set, Ser>(
    uid: Uid,
    set: &mut Set,
    ser: &mut Ser,
    raw: Option<&RawVal>,
    val: Option<i64>,
) -> Result<bool, CoteError>
where
    Set: SetValueFindExt,
{
    let mut act = *set.opt_mut(uid)?.action();

    act.process(uid, set, ser, raw, val.map(|v| v + 2))
}
