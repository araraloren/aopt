use cote::prelude::*;
use std::time::{Duration, SystemTime};

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    // Using `index` and `append` collect the position arguments 1..
    #[pos(index = 1.., append, fetch = random_select)]
    name: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let names = ["lily", "lucy", "bob", "joe"];
    let cli = Cli::parse(Args::from(["app"].into_iter().chain(names.into_iter())))?;

    assert!(names.contains(&cli.name.as_str()));
    Ok(())
}

// The fetch handler which is used to extract value from `set`
fn random_select<T: ErasedTy + Default>(
    name: &str,
    set: &mut impl SetValueFindExt,
) -> Result<T, CoteError> {
    let vals = set.find_vals_mut::<T>(name)?;
    let len = vals.len();
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    let now = now.unwrap_or(Duration::from_secs(1));

    Ok(std::mem::replace(
        &mut vals[now.as_secs() as usize % len],
        T::default(),
    ))
}
