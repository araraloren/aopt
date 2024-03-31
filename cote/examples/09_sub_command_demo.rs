use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Cli {
    #[arg()]
    bar: usize,

    #[sub(alias = "z")]
    baz: Option<Baz>,

    #[sub(alias = "x")]
    qux: Option<Qux>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Baz {
    grault: bool,

    waldo: Option<String>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Qux {
    garply: bool,

    fred: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse(Args::from(["app", "--bar=42", "z"]))?;

    assert_eq!(cli.bar, 42);
    assert_eq!(
        cli.baz,
        Some(Baz {
            grault: false,
            waldo: None
        })
    );
    assert_eq!(cli.qux, None);

    let cli = Cli::parse(Args::from(["app", "--bar=42", "x", "--fred", "plugh"]))?;

    assert_eq!(cli.bar, 42);
    assert_eq!(cli.baz, None);
    assert_eq!(
        cli.qux,
        Some(Qux {
            garply: false,
            fred: "plugh".to_owned()
        })
    );

    Ok(())
}
