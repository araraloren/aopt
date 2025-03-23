use cote::prelude::*;
use std::{ffi::OsStr, fmt::Debug};

// The handler must be a generic function.
#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, on = display_cli)]
pub struct Cli {
    #[arg(on = empty_handler, then = foo_storer)]
    foo: u64,

    #[sub(force = false)]
    bar: Option<Bar>,

    #[sub(force = false)]
    qux: Option<Qux>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Bar {
    #[arg(force = false, fallback = debug_of_bar)]
    debug: bool,

    #[pos()]
    quux: String,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, fallback = process_qux, then = unreachable_storer)]
pub struct Qux {
    #[cmd(name = "c")]
    corge: bool,

    grault: Option<i64>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // unwrap the failure of return value
    Cli::parse_env_args()?.ret.unwrap();

    Ok(())
}

fn display_cli<Set>(set: &mut Set, _: &mut Ctx) -> Result<Option<()>, aopt::Error>
where
    Set: SetValueFindExt + cote::prelude::Set,
    SetCfg<Set>: ConfigValue + Default,
{
    println!("Got client: {:?}", Cli::try_extract(set)?);
    Ok(None)
}

fn empty_handler<Set>(_: &mut Set, ctx: &mut Ctx) -> Result<Option<u64>, aopt::Error> {
    Ok(ctx.value::<u64>().ok())
}

fn foo_storer<Set>(
    uid: Uid,
    set: &mut Set,
    raw: Option<&OsStr>,
    val: Option<u64>,
) -> Result<bool, aopt::Error>
where
    Set: SetValueFindExt + cote::prelude::Set,
    SetCfg<Set>: ConfigValue + Default,
{
    let has_value = val.is_some();

    // Set the value if return Some(Value)
    if let Some(val) = val {
        if let Some(opt) = set.get_mut(uid) {
            let (raw_handler, handler) = opt.accessor_mut().handlers();

            if let Some(raw_value) = raw {
                raw_handler.push(raw_value.to_os_string());
            }
            println!("Saving the value of `--foo` to {}", val + 1);
            // modify the value, plus one
            handler.push(val + 1);
        }
    }

    Ok(has_value)
}

fn debug_of_bar<Set>(_: &mut Set, ctx: &mut Ctx) -> Result<Option<()>, aopt::Error> {
    let raw = ctx.arg()?.unwrap();
    let value = ctx.value::<bool>()?;

    println!("Got value of `--debug`: {:?} --> {}", raw, value);
    // if return None, the parser will call default handler of current option
    Ok(None)
}

fn process_qux<Set>(_: &mut Set, _: &mut Ctx) -> Result<Option<()>, aopt::Error>
where
    Set: SetValueFindExt + cote::prelude::Set,
    SetCfg<Set>: ConfigValue + Default,
{
    println!("return Ok(None) call the default handler of Qux");
    Ok(None)
}

fn unreachable_storer<Set>(
    _: Uid,
    _: &mut Set,
    _: Option<&OsStr>,
    _: Option<()>,
) -> Result<bool, aopt::Error>
where
    Set: SetValueFindExt + cote::prelude::Set,
    SetCfg<Set>: ConfigValue + Default,
{
    unreachable!("Never go here")
}
