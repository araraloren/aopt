use std::sync::Mutex;

use cote::*;

static GLOBAL_CNT: Mutex<i32> = Mutex::new(0);

macro_rules! order {
    ($n:literal, $t:ident) => {
        |_: &mut Parser<'_, Set, Ser>, _: &mut Ser, mut val: ctx::Value<$t>| {
            *GLOBAL_CNT.lock().unwrap() += 1;
            assert_eq!($n, *GLOBAL_CNT.lock().unwrap());
            Ok(Some(val.take()))
        }
    };
}

#[derive(Debug, Cote)]
#[cote(help)]
pub struct Cli {
    #[allow(unused)]
    debug: bool,

    #[allow(unused)]
    #[sub(policy = delay)]
    query: Option<Query>,
}

#[derive(Debug, Cote)]
pub struct Query {
    #[allow(unused)]
    #[arg(nodelay, on = order!(1, usize))]
    row: usize, // `nodelay` option will be process immediately, 
                // before `col` and `format`

    #[allow(unused)]
    #[arg(on = order!(3, usize))]
    col: usize, // `col` is process after `format` when using `DelayPolicy`

    #[allow(unused)]
    #[pos(on = order!(2, String))]
    format: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse(Args::from(
        ["app", "query", "--col=1", "--row=2", "Query msg: {:?}"].into_iter(),
    ))?;
    assert_eq!(*GLOBAL_CNT.lock().unwrap(), 3);
    Ok(())
}
