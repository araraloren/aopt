use cote::prelude::*;
use std::time::{Duration, SystemTime};

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    // Using `index` and `append` collect the position arguments 1..
    #[pos(index = 1.., append, fetch = random_select)]
    name: String,
}

#[test]
fn fetch() {
    assert!(fetch_impl().is_ok());
}

fn fetch_impl() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let names = ["lily", "lucy", "bob", "joe"];
    let cli = Cli::parse(Args::from(["app"].into_iter().chain(names.into_iter())))?;

    assert!(names.contains(&cli.name.as_str()));
    Ok(())
}

// The fetch handler which is used to extract value from `set`
fn random_select<T, S>(uid: Uid, set: &mut S) -> cote::Result<T>
where
    T: ErasedTy + Default,
    S: Set + SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
    let vals = set.opt_mut(uid)?.vals_mut::<T>()?;
    let len = vals.len();
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    let now = now.unwrap_or(Duration::from_secs(1));

    Ok(std::mem::take(&mut vals[now.as_secs() as usize % len]))
}
