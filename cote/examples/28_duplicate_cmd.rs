use cote::prelude::*;

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct Cli {
    #[sub(name = "w")]
    widget_opt: Option<WidgetOpt>,

    #[sub(name = "w")]
    widget_pos: Option<WidgetPos>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct WidgetOpt {
    cnt: i64,

    name: String,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct WidgetPos {
    #[pos()]
    cnt: i64,

    #[pos()]
    name: String,
}

fn main() -> color_eyre::Result<()> {
    //Result<(), aopt::Error> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from(["app", "w", "--cnt=32", "--name", "lily"]))?;

    assert!(cli.widget_opt.is_some());

    assert_eq!(
        cli.widget_opt,
        Some(WidgetOpt {
            cnt: 32,
            name: String::from("lily")
        })
    );

    let cli = Cli::parse(Args::from(["app", "w", "32", "lily"]))?;

    assert!(cli.widget_pos.is_some());

    assert_eq!(
        cli.widget_pos,
        Some(WidgetPos {
            cnt: 32,
            name: String::from("lily")
        })
    );

    Ok(())
}
