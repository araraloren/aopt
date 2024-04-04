use std::ops::Add;

use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    #[arg(on = plus_one, then = plus_two)]
    value: i64,
}

#[test]
fn on() {
    assert!(on_impl().is_ok());
}

fn on_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse(Args::from(["app", "--value=39"].into_iter()))?;
    assert_eq!(cli.value, 42);
    Ok(())
}

fn plus_one<Set, Ser>(
    _: &mut Set,
    _: &mut Ser,
    val: ctx::Value<i64>, // extract value from argument
) -> cote::Result<Option<i64>> {
    Ok(Some(val.add(1)))
}

fn plus_two<Set, Ser>(
    uid: Uid,
    set: &mut Set,
    ser: &mut Ser,
    raw: Option<&RawVal>,
    val: Option<i64>,
) -> cote::Result<bool>
where
    Set: SetValueFindExt,
    SetCfg<Set>: ConfigValue + Default,
{
    let mut act = *set.opt_mut(uid)?.action();

    act.process(uid, set, ser, raw, val.map(|v| v + 2))
}
