# aopt

A flexible and typed getopt like command line framwork for rust.

## Features

- Option support

    - Prefixed option support, such as `-f`, `--flag`, `-flag` or `--/flag`.

    - Option value support, such as `-f 42`, `--flag 3.14` or `--flag=foo`.

    - Multiple style option support, such as `-f 42`, `-f=42` or `-f42`.

    - Combing style support, such as `-abc` is same as `-a` `-b` `-c`.

    - Positional arguments support, see [`Index`](crate::opt::Index).

    - Type support, you can validator the value of option during parsing.

    See the built-in option type [`AOpt`](crate::opt::AOpt)

- Non UTF8 arguments support

- Callback support

    Can set callback which will called during parsing,
    see [`Parser`](crate::parser::Parser) and [`Invoker`](crate::ctx::Invoker).

- Value support

    By default aopt will saving the raw value and parsed value in [`ValStorer`](crate::value::ValStorer).

- Policy support

    - [`DelayPolicy`](crate::parser::DelayPolicy) process positional arguments before any other option.

    - [`FwdPolicy`](crate::parser::FwdPolicy) process options before positional arguments.

    - [`PrePolicy`](crate::parser::PrePolicy) can help you process the options partial.

- Derive support

    - Checkout [`cote`](https://docs.rs/cote/latest/cote/index.html) crate for derive support and help message generate.

## Setup

Add following to your `Cargo.toml` file:

```toml
[dependencies]
aopt = "0.9"
```

### `sync` feature

If you want the utils of current crate implement `Send` and `Sync`, you can enable `sync` feature.

### `utf8` feature

By default, the command line parsing support `OsString`, enable `utf8` using `String` instead.

## Simple flow chart

```txt
                     +---------------------------------------+
                     |             Policy                    |
                     |                                       |
+--------------+     |  +-----------+     +------------+     |                +-------------+
|              |     |  |           |     |            |     |   Invoke       |             |
|   Arguments  +---->|  |  Checker  |     |   Process  |<----+----------------+   Invoker   |
|              |     |  |           |     |            |     |   the callback |             |
+--------------+     |  +---^-------+     ++-----^-----+     |                +-------------+
                     |      |              |     |           |
                     |      |              |     |           |
                     +------+--------------+-----+-----------+
                            |              |     |
                            |              |     |
                            |  Save the values   |Process the arguments
                            |              |     |
                            |              |     |
                Check the options          |     |
                            |              |     |
                            |              |     |
                            |         +----v-----+-----------+
                            |         |                      |
                            +---------+      Option Set      |
                                      |                      |
                                      +----------------------+
```

## Example

- Using [`AFwdParser`](crate::ext::AFwdParser) parsing process the command line.

```no_run
use aopt::prelude::*;
use std::ops::Deref;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = AFwdParser::default();

    parser.validator_mut().add_prefix("+");
    parser.add_opt("--depth=i")?.set_value_t(0i64); // int option
    parser.add_opt("-/r=b")?; // boolean flag
    parser
        .add_opt("--source=s!")? // ! means the option is force required
        .add_alias("+S")
        .on(
            |set: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| {
                let depth: &i64 = set["--depth"].val()?;
                println!("Adding location({}) with depth({})", val.deref(), depth);
                Ok(Some((val.take(), *depth)))
            },
        )?;
    parser.add_opt("destination=p!@-0")?.on(
        |_: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| {
            println!("Save destination location({})", val.deref());
            Ok(Some(val.take()))
        },
    )?;
    parser.add_opt("main=m")?.on(
        |set: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| {
            let src = set["--source"].vals::<(String, i64)>()?;
            let dest: &String = set["destination"].val()?;

            for (item, depth) in src {
                println!(
                    "Application {} will copy location({item}, depth={depth}) to destination({})",
                    val.deref(),
                    dest
                );
            }
            Ok(Some(val.take()))
        },
    )?;
    parser.init()?;
    parser.parse_env()?.unwrap();

    Ok(())
}
```

- Using [`getopt!`](crate::getopt) parsing multiple sub command.

```no_run
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
    list.add_opt_i::<String>("-source")?
        .add_alias("-s")
        .set_value(String::from("lib.rs"));
    list.add_opt("main=m")?
        .fallback(|set: &mut ASet, _: &mut ASer| {
            println!(
                "invoke list command: debug={:?}, force={:?}, local-only={:?}, source={:?}",
                set["-debug"].val::<bool>()?,
                set["-force"].val::<bool>()?,
                set["-local-only"].val::<bool>()?,
                set["-source"].val::<String>()?,
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
        .on(|set: &mut ASet, _: &mut ASer| {
            println!(
                "invoke update command: debug={:?}, force={:?}, source={:?}",
                set["-debug"].val::<bool>()?,
                set["-force"].val::<bool>()?,
                set["-source"].val::<String>()?,
            );
            Ok(Some(true))
        })?;

    install.add_opt("install=c")?;
    install.add_opt("in=c")?;
    install.add_opt("-debug=b")?;
    install.add_opt("-/override=b")?.add_alias("-/o");
    install.add_opt("-source=s")?.add_alias("-s");
    install.add_opt("name=p!@2")?.on(
        |set: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| {
            if val.deref() == "software" {
                println!(
                    "invoke install command: debug={:?}, override={:?}, source={:?}",
                    set["-debug"].val::<bool>()?,
                    set["-/override"].val::<bool>()?,
                    set["-source"].val::<String>()?,
                );
                Ok(Some(val.take()))
            } else {
                Err(Error::raise_error("command not matched"))
            }
        },
    )?;

    getopt!(Args::from_env(), &mut list, &mut update, &mut install)?;
    Ok(())
}
```

* `app.exe ls -debug` output

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