use cote::prelude::*;

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Cli {
    #[arg(alias = "-g")]
    age: usize,

    /// Help message of eat sub command
    #[sub()]
    eat: Option<Eat>,

    /// Help message of sport sub command
    #[sub(policy = pre)]
    sport: Option<Sport>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Eat {
    /// Which meal did you have?
    #[arg(alias = "-m")]
    meal: String,

    /// What did you wat?
    #[pos(value = "rice")]
    what: Option<String>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Sport {
    /// Go for a walk.
    #[sub()]
    walk: Option<Walk>,

    /// Play some games.
    #[sub()]
    play: Option<Play>,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Walk {
    #[arg(name = "-d", value = 3usize)]
    distance: usize,
}

#[derive(Debug, Cote, PartialEq, Eq)]
#[cote(help, aborthelp)]
pub struct Play {
    /// Which game do you want to play?
    #[pos(value = "Mario")]
    game: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse_env()?;

    println!("You age is set to {}", cli.age);
    if let Some(eat) = cli.eat {
        println!("You {} are going to eat {}", eat.meal, eat.what.unwrap());
    } else if let Some(sport) = cli.sport {
        if let Some(walk) = sport.walk {
            println!("You are going to walk {} kilometers", walk.distance);
        } else if let Some(play) = sport.play {
            println!("You are going to play game {}", play.game);
        }
    }
    Ok(())
}
