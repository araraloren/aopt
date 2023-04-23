use cote::prelude::*;
use std::{fmt::Debug, ops::Deref};

// The handler must be a generic function.
#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, on = display_cli::<P>)]
pub struct Cli {
    #[arg(on = empty_handler::<P>, then = foo_storer::<P>)]
    foo: u64,

    #[sub(force = false)]
    bar: Option<Bar>,

    #[sub(force = false)]
    qux: Option<Qux>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Bar {
    #[arg(force = false, fallback = debug_of_bar::<P>)]
    debug: bool,

    #[pos()]
    quux: String,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, fallback = process_qux::<P>, then = unreachable_storer::<P>)]
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

fn display_cli<P>(set: &mut P::Set, _: &mut P::Ser) -> Result<Option<()>, aopt::Error>
where
    P: Policy,
    P::Set: SetValueFindExt + Set,
{
    println!("Got client: {:?}", Cli::try_extract(set)?);
    Ok(None)
}

fn empty_handler<P>(
    _: &mut P::Set,
    _: &mut P::Ser,
    value: Option<ctx::Value<u64>>,
) -> Result<Option<u64>, aopt::Error>
where
    P: Policy,
{
    Ok(value.map(|mut v| v.take()))
}

fn foo_storer<P>(
    uid: Uid,
    set: &mut P::Set,
    _: &mut P::Ser,
    raw: Option<&RawVal>,
    val: Option<u64>,
) -> Result<bool, aopt::Error>
where
    P: Policy,
    P::Set: SetValueFindExt + Set,
{
    let has_value = val.is_some();

    // Set the value if return Some(Value)
    if let Some(val) = val {
        if let Some(opt) = set.get_mut(uid) {
            let (raw_handler, handler) = opt.accessor_mut().handlers();

            if let Some(raw_value) = raw {
                raw_handler.push(raw_value.clone());
            }
            println!("Saving the value of `--foo` to {}", val + 1);
            // modify the value, plus one
            handler.push(val + 1);
        }
    }

    Ok(has_value)
}

fn debug_of_bar<P>(
    _: &mut P::Set,
    _: &mut P::Ser,
    raw: ctx::RawVal,
    value: ctx::Value<bool>,
) -> Result<Option<()>, aopt::Error>
where
    P: Policy,
{
    println!(
        "Got value of `--debug`: {:?} --> {}",
        raw.deref(),
        value.deref()
    );
    // if return None, the parser will call default handler of current option
    Ok(None)
}

fn process_qux<P>(_: &mut P::Set, _: &mut P::Ser) -> Result<Option<()>, aopt::Error>
where
    P: Policy,
    P::Set: SetValueFindExt + Set,
{
    println!("return Ok(None) call the default handler of Qux");
    Ok(None)
}

fn unreachable_storer<P>(
    _: Uid,
    _: &mut P::Set,
    _: &mut P::Ser,
    _: Option<&RawVal>,
    _: Option<()>,
) -> Result<bool, aopt::Error>
where
    P: Policy,
    P::Set: SetValueFindExt + Set,
{
    unreachable!("Never go here")
}
