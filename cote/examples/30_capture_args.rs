use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct Cli {
    opt: String,

    cnt: Option<i32>,

    _useless_field: Option<Stop>,

    // you need specify the index of positional type
    #[pos(index = 1..)]
    args: Option<Vec<String>>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from([
        "app",
        "--opt=val",
        "--cnt",
        "32",
        "--",
        "--opt=bar",
        "lily",
    ]))?;

    assert_eq!(cli.opt.as_str(), "val");
    assert_eq!(cli.cnt, Some(32));
    assert_eq!(cli._useless_field, Some(Stop));
    assert_eq!(
        cli.args,
        Some(["--opt=bar", "lily"].map(String::from).to_vec())
    );

    let cli = Cli::parse(Args::from([
        "app",
        "--opt=val",
        "--",
        "--cnt=64",
        "--opt=bar",
        "lily",
    ]))?;

    assert_eq!(cli.opt.as_str(), "val");
    assert_eq!(cli.cnt, None);
    assert_eq!(cli._useless_field, Some(Stop));
    assert_eq!(
        cli.args,
        Some(["--cnt=64", "--opt=bar", "lily"].map(String::from).to_vec())
    );

    let cli = Cli::parse(Args::from(["app", "--opt=val", "--cnt", "128"]))?;

    assert_eq!(cli.opt.as_str(), "val");
    assert_eq!(cli.cnt, Some(128));
    assert_eq!(cli._useless_field, None);
    assert_eq!(cli.args, None);

    Ok(())
}
