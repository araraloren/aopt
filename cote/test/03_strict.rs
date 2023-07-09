use cote::*;

#[derive(Debug, Cote)]
#[cote(help, strict = true)]
pub struct Cli;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Cli::parse(Args::from(["app", "--opt-a"].into_iter()))?;
    // Output:
    // 
    // Error:
    //    0: Parsing arguments `"--opt-a"` failed: None
    //    1: Can not find option `--opt-a`
    //
    // Location:
    //    src/main.rs:9
    //
    // Backtrace omitted.
    Ok(())
}
