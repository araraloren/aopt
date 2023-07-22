use cote::*;

#[derive(Debug, Cote)]
#[cote()]
pub struct Cli {
    #[pos()]
    name: String,

    #[pos(index = 2..)]
    args: Vec<u64>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse(Args::from(["app", "index", "2", "3", "4"].into_iter()))?;
    assert_eq!(cli.name.as_str(), "index");
    assert_eq!(cli.args, vec![2, 3, 4]);
    Ok(())
}
