use cote::prelude::*;

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

fn cmd_order<S>(set: &mut Parser<'_, S>, _: &mut Ctx) -> Result<Option<bool>, aopt::Error> {
    let order = set.app_data_mut::<usize>()?;
    *order += 1;
    let order = *order;
    assert_eq!(order, 2);
    println!("Order {}", order);
    Ok(Some(true))
}

fn assert_order<S>(set: &mut Parser<'_, S>, ctx: &mut Ctx) -> Result<Option<usize>, aopt::Error> {
    let order = set.app_data_mut::<usize>()?;
    *order += 1;
    let order = *order;
    println!("Order {}", order);
    let val = ctx.value::<usize>()?;

    assert_eq!(order, val);
    Ok(Some(val))
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut app = Cli::into_parser()?;
    let mut policy = Cli::into_policy();

    app.set_app_data(0usize);
    app.run_mut_with(
        ["app", "foo", "--bar=4", "--qux=1", "3"].into_iter(),
        &mut policy,
        |_, app| {
            let cli = Cli::try_extract(app.optset_mut())?;
            assert!(cli.foo);
            assert_eq!(cli.bar, 4);
            assert_eq!(cli.qux, 1);
            assert_eq!(cli.baz, 3);
            Ok(())
        },
    )?;

    Ok(())
}
