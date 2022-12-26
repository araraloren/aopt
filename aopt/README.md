# aopt

A flexible and typed getopt like command line tools for rust.

## Setup

Add following to your `Cargo.toml` file:

```toml
[dependencies]
aopt = "0.8"
```

### Enable `sync` feature

If you want the utils of current crate implement `Send` and `Sync`, you can enable `sync` feature.

```toml
[dependencies]
aopt = { version = "0.8", features = [ "sync" ] }
```

### Enable `utf8` feature

By default, the command line parsing support `OsString`, enable `utf8` using `String` instead.

```toml
[dependencies]
aopt = { version = "0.8", features = [ "utf8" ] }
```

## Example

With `getopt!` and `Parser`, you can match and process every command easily.

```ignore
use aopt::prelude::*;
use aopt::Error;
use std::ops::Deref;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut list = AFwdParser::default();
    let mut update = AFwdParser::default();
    let mut install = AFwdParser::default();

    list.add_opt("list=c")?;
    list.add_opt("ls=c")?;
    list.add_opt("-debug=b")?;
    list.add_opt("-force=b")?.add_alias("-f");
    list.add_opt("-local-only=b")?.add_alias("-l");
    list.add_opt("-source=s")?
        .add_alias("-s")
        .set_value(String::from("lib.rs"));
    list.add_opt("main=m")?
        .fallback(|set: &mut ASet, ser: &mut ASer| {
            println!(
                "invoke list command: debug={:?}, force={:?}, local-only={:?}, source={:?}",
                ser.sve_val::<bool>(set["-debug"].uid())?,
                ser.sve_val::<bool>(set["-force"].uid())?,
                ser.sve_val::<bool>(set["-local-only"].uid())?,
                ser.sve_val::<String>(set["-source"].uid())?,
            );
            Ok(None::<()>)
        })?;

    update.add_opt("update=c")?;
    update.add_opt("up=c")?;
    update.add_opt("-debug=b")?;
    update.add_opt("-force=b")?.add_alias("-f");
    update.add_opt("-source=s")?.add_alias("-s");
    update
        .add_opt("main=m")?
        .on(|set: &mut ASet, ser: &mut ASer| {
            println!(
                "invoke update command: debug={:?}, force={:?}, source={:?}",
                ser.sve_val::<bool>(set["-debug"].uid())?,
                ser.sve_val::<bool>(set["-force"].uid())?,
                ser.sve_val::<String>(set["-source"].uid())?,
            );
            Ok(Some(true))
        })?;

    install.add_opt("install=c")?;
    install.add_opt("in=c")?;
    install.add_opt("-debug=b")?;
    install.add_opt("-/override=b")?.add_alias("-/o");
    install.add_opt("-source=s")?.add_alias("-s");
    install.add_opt("name=p!@2")?.on(
        |set: &mut ASet, ser: &mut ASer, mut val: ctx::Value<String>| {
            if val.deref() == "software" {
                println!(
                    "invoke install command: debug={:?}, override={:?}, source={:?}",
                    ser.sve_val::<bool>(set["-debug"].uid())?,
                    ser.sve_val::<bool>(set["-/override"].uid())?,
                    ser.sve_val::<String>(set["-source"].uid())?,
                );
                Ok(Some(val.take()))
            } else {
                Err(Error::raise_error("command not matched"))
            }
        },
    )?;

    getopt!(
        std::env::args().skip(1),
        &mut list,
        &mut update,
        &mut install
    )?;
    Ok(())
}
```

* `app.exe ls -source lib.rs -debug` output 

    invoke list command: debug=true, force=false, local-only=false, source="lib.rs"

* `app.exe update -force -source=crates.io` output

    invoke update command: debug=false, force=true, source="crates.io"

* `app.exe in software -/o -s crates.io` output

    invoke install command: debug=false, override=true, source=Str("crates.io")

* `app.exe in aopt` output

    Error: command not matched

## More

- simple-find-file

A simple file search tools, try it using [`cargo install --path simple-find-file`](https://github.com/araraloren/aopt/tree/main/simple-find-file).

- snowball-follow

Get the follow count of stock in `xueqiu.com`, try it using [`cargo install --path snowball-follow`](https://github.com/araraloren/aopt/tree/main/snowball-follow)

- index constituent

Search and list the constituent of index, try it using [`cargo install --path index-constituent`](https://github.com/araraloren/aopt/tree/main/index-constituent)

## Release log

Follow the [link](https://github.com/araraloren/aopt/blob/main/aopt/Release.md).

## LICENSE

MPL-2.0