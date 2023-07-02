use cote::*;
use std::ops::Deref;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(policy = delay, help)]
pub struct Cli {
    #[cmd(on = cmd_order)]
    foo: bool,

    #[arg(on = assert_order)]
    bar: usize,

    #[pos(on = assert_order, index = 2)]
    baz: usize,

    #[arg(on = assert_order, nodelay)]
    qux: usize,
}

fn cmd_order<Set, Ser>(_: &mut Set, ser: &mut Ser) -> Result<Option<bool>, aopt::Error>
where
    Ser: ServicesValExt,
{
    let order = ser.sve_val_mut::<usize>()?;
    *order += 1;
    let order = *order;
    assert_eq!(order, 2);
    println!("Order {}", order);
    Ok(Some(true))
}

fn assert_order<Set, Ser>(
    _: &mut Set,
    ser: &mut Ser,
    mut val: ctx::Value<usize>,
) -> Result<Option<usize>, aopt::Error>
where
    Ser: ServicesValExt,
{
    let order = ser.sve_val_mut::<usize>()?;
    *order += 1;
    let order = *order;
    println!("Order {}", order);
    assert_eq!(order, *val.deref());
    Ok(Some(val.take()))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut app = Cli::into_parser()?;
    let mut policy = Cli::into_policy();

    app.set_app_data(0usize)?;
    app.run_mut_with(
        ["app", "foo", "--bar=4", "--qux=1", "3"].into_iter(),
        &mut policy,
        |_, app| {
            let cli = Cli::try_extract(app.optset_mut())?;
            assert_eq!(cli.foo, true);
            assert_eq!(cli.bar, 4);
            assert_eq!(cli.qux, 1);
            assert_eq!(cli.baz, 3);
            Ok(())
        },
    )?;

    Ok(())
}
