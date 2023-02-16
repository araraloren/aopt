
# Cote

A simple option manager manage the [`AOpt`](aopt::opt::AOpt), support auto generate help message.

## Setup

Add following to your `Cargo.toml` file:

```toml
[dependencies]
cote = "0.2"
```

## Enable Features from aopt

### Enable `sync` feature

If you want the utils of current crate implement `Send` and `Sync`, you can enable `sync` feature.

```toml
[dependencies]
cote = { version = "0.2", features = [ "sync" ] }
```

### Enable `utf8` feature

By default, the command line parsing support `OsString`, enable `utf8` using `String` instead.

```toml
[dependencies]
cote = { version = "0.2", features = [ "utf8" ] }
```

## Example

```ignore
use cote::prelude::*;
use aopt::prelude::*;
use aopt::Error;

fn main() -> Result<(), Error> {
    let mut cote = Cote::<AFwdPolicy>::default();

    cote.add_meta::<String>(
        serde_json::from_str(
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
    cote.insert_help(env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_DESCRIPTION"))?;
    cote.add_opt("--from=i")?
        .add_alias("-f")
        .set_help("The sub string start index");
    cote.add_opt("--to=i")?
        .add_alias("-t")
        .set_help("The sub string end index");
    cote.run_with(
        ["-f", "5", "-t", "9"].into_iter(),
        |ret, cote: &Cote<AFwdPolicy>| {
            if ret.is_some() {
                let start: i64 = *cote.find_val("--from")?;
                let end: i64 = *cote.find_val("--to")?;

                println!(
                    "cote running okay: {:?}",
                    cote.find_val::<String>("-s")?
                        .get(start as usize..end as usize)
                );
            }
            Ok(())
        },
    )?;

    Ok(())
}
```

## LICENSE

MPL-2.0