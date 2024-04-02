
# Cote

A simple option manager manage the [`AOpt`](aopt::opt::AOpt), support auto generate help message.

## Setup

`cargo add cote` or add following to your `Cargo.toml` file:

```toml
[dependencies]
cote = "0.4"
```

## Enable Features from aopt

### Enable `sync` feature

If you want the utils of current crate implement `Send` and `Sync`, you can enable `sync` feature.

```toml
[dependencies]
cote = { version = "*", features = [ "sync" ] }
```

## Documents 

See [`reference`](crate::_reference) for more information.

### Example

#### Using [`Cote`](crate::cote_derive::Cote) generate struct from command line options.

```rust
use aopt::opt::Pos;
use cote::prelude::*;

fn main() -> cote::Result<()> {
    #[derive(Debug, Cote)]
    pub struct Cli {
        /// A flag option named `--flag`
        flag: bool,

        /// Comment here set the help message for option
        #[arg(alias = "-n")]
        name: String,

        #[arg(help = "`Option` mean the option is not force required")]
        nick: Option<String>,

        /// A position option at index 1
        #[arg(index = "1")]
        from: Pos<String>,

        /// A positon option collect argument start from 2
        #[pos(index = 2..)]
        to: Vec<String>,
    }
    let cli = Cli::parse(Args::from(["app", "-nLily", "src", "foo", "bar"]))?;

    assert!(!cli.flag);
    assert_eq!(cli.name, String::from("Lily"));
    assert_eq!(cli.nick, None);
    assert_eq!(cli.from, Pos(String::from("src")));
    assert_eq!(cli.to, vec![String::from("foo"), String::from("bar")]);

    let cli = Cli::parse(Args::from(["app", "--name", "Lily", "src", "foo", "bar"]))?;

    assert!(!cli.flag);
    assert_eq!(cli.name, String::from("Lily"));
    assert_eq!(cli.nick, None);
    assert_eq!(cli.from, Pos(String::from("src")));
    assert_eq!(cli.to, vec![String::from("foo"), String::from("bar")]);

    assert!(Cli::parse(Args::from(["app", "--nick", "Lily", "src", "foo", "bar"])).is_err());

    Ok(())
}
```

## LICENSE

MPL-2.0