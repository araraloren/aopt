use std::ffi::OsStr;

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

fn plus_one<S>(_: &mut S, ctx: &mut Ctx) -> cote::Result<Option<i64>> {
    Ok(Some(ctx.value::<i64>()? + 1))
}

fn plus_two<S>(uid: Uid, set: &mut S, raw: Option<&OsStr>, val: Option<i64>) -> cote::Result<bool>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
    let mut act = *set.opt_mut(uid)?.action();

    act.process(uid, set, raw, val.map(|v| v + 2))
}
