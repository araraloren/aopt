
# Cote

A simple option manager manage the [`AOpt`](aopt::opt::AOpt), support auto generate help message.

## Setup

Add following to your `Cargo.toml` file:

```toml
[dependencies]
cote = "0.3"
```

## Enable Features from aopt

### Enable `sync` feature

If you want the utils of current crate implement `Send` and `Sync`, you can enable `sync` feature.

```toml
[dependencies]
cote = { version = "0.3", features = [ "sync" ] }
```

### Enable `utf8` feature

By default, the command line parsing support `OsString`, enable `utf8` using `String` instead.

```toml
[dependencies]
cote = { version = "0.3", features = [ "utf8" ] }
```

## Example

### Using [`Cote`](crate::cote_derive::Cote) generate struct from command line options.

```rust
use aopt::opt::Pos;
use aopt::Error;
use cote::prelude::*;

fn main() -> Result<(), Error> {
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
        #[pos(index = "2..")]
        to: Vec<String>,
    }
    let cli = Cli::parse(Args::from_array(["app", "-nLily", "src", "foo", "bar"]))?;

    assert_eq!(cli.flag, false);
    assert_eq!(cli.name, String::from("Lily"));
    assert_eq!(cli.nick, None);
    assert_eq!(cli.from, Pos(String::from("src")));
    assert_eq!(cli.to, vec![String::from("foo"), String::from("bar")]);

    let cli = Cli::parse(Args::from_array(["app", "--name", "Lily", "src", "foo", "bar"]))?;

    assert_eq!(cli.flag, false);
    assert_eq!(cli.name, String::from("Lily"));
    assert_eq!(cli.nick, None);
    assert_eq!(cli.from, Pos(String::from("src")));
    assert_eq!(cli.to, vec![String::from("foo"), String::from("bar")]);

    assert!(Cli::parse(Args::from_array(["app", "--nick", "Lily", "src", "foo", "bar"])).is_err());

    Ok(())
}
```

See [`tutorial`](crate::_reference) for more information.

### Using [`CoteApp`](crate::CoteApp) load option from json configuration.

```rust
use cote::prelude::*;
use aopt::Error;

fn main() -> Result<(), Error> {
    let mut cote = CoteApp::<AFwdPolicy>::default();

    // load option from json
    cote.add_opt_meta(
        serde_json::from_str::<'_, OptionMeta<String>>(
            r#"
                    {
                        "id": "s",
                        "option": "-s=s",
                        "hint": "-s <str>",
                        "help": "This is a help for option [-s]",
                        "value": [
                        "cote manager"
                        ]
                    }
                    "#,
        )
        .unwrap(),
    )?;
    cote.add_help_option()?;
    cote.add_opt("--from=i")?
        .add_alias("-f")
        .set_help("The sub string start index");
    cote.add_opt("--to=i")?
        .add_alias("-t")
        .set_help("The sub string end index");
    cote.run_mut_with(
        ["-f", "5", "-t", "9"].into_iter(),
        |_, cote: &mut CoteApp<AFwdPolicy>| {
            if display_help!(cote)? {
                std::process::exit(0);
            }
            #[derive(Debug, Cote)]
            pub struct Setting {
                from: i64,

                to: i64,

                #[arg(name = "-s")]
                string: String,
            }

            // You can extract the type from CoteApp
            let setting = cote.extract_type::<Setting>()?;

            assert_eq!(setting.from, 5);
            assert_eq!(setting.to, 9);
            assert_eq!(setting.string, String::from("cote manager"));
            Ok(())
        },
    )?;
    Ok(())
}
```

## LICENSE

MPL-2.0