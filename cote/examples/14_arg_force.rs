use cote::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help)]
pub struct Cli {
    // `cmd` is force required in default, you can't change it
    #[cmd()]
    foo: bool,

    // `Option` make the `pos` optional in default
    #[pos(index = "2", value = 42usize)]
    bar: Option<usize>,

    // Without `Option`, `--baz` is force required
    #[arg(alias = "-b", help = "Set the string value of baz")]
    baz: String,

    // Using `force` you can force set the option to force required
    #[arg(force = true)]
    qux: Option<i64>,

    // If the option has default value, then it is optional
    #[arg(values = ["need"])]
    quux: Vec<String>,
}
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    assert!(Cli::parse(Args::from_array(["app", "--baz=6"])).is_err());

    assert!(Cli::parse(Args::from_array(["app", "foo", "--baz=6"])).is_err());

    assert!(Cli::parse(Args::from_array(["app", "--qux", "-5", "foo", "--baz=6"])).is_ok());

    Ok(())
}
