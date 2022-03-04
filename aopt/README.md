# aopt

A flexible and typed getopt like command line tools for rust.

## Example1

`SingleApp` can help you create a simple tools command.

```rust
use aopt::prelude::*;
use aopt::SingleApp;
use aopt::Result;

fn main() -> Result<()> {
    let mut single_app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default()
        .with_name("example".into());

    // default prefix of SimpleSet is '-' and '--'
    single_app.add_prefix("+".into());
    single_app
        .add_opt("--depth=i")?
        .set_help("set the search depth of directory")
        .commit()?;
    single_app
        .add_opt("-r=b/")?
        .set_help("disable recurse directory option")
        .commit()?;

    fn display_depth_and_source(
        _: Uid,
        set: &mut SimpleSet,
        value: OptValue,
    ) -> aopt::err::Result<Option<OptValue>> {
        let depth = set.get_value("--depth")?.unwrap().as_int().unwrap_or(&0);
        let current = value.as_vec().unwrap().last().unwrap();
        println!("Adding location({}) with depth({})", current, depth);
        Ok(Some(value))
    }
    let srcid = single_app
        .add_opt_cb("--source=a!", simple_opt_mut_cb!(display_depth_and_source))?
        .add_alias("+S")?
        .set_help("add search source directory")
        .commit()?;

    single_app.add_opt("--debug=b")?.commit()?;
    single_app
        .add_opt_cb(
            "destination=p!@-1",
            simple_pos_cb!(move |_, set: &SimpleSet, arg, _, _| {
                let mapped: Vec<(String, String)> = set[srcid]
                    .get_value()
                    .as_vec()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        println!("Save copy location({}) to destination({})", v, arg);
                        (v.clone(), arg.into())
                    })
                    .collect();
                // you can even keep Any to OptValue
                Ok(Some(OptValue::from_any(Box::new(mapped))))
            }),
        )?
        .commit()?;

    single_app.run_mut(&mut std::env::args().skip(1), |ret, app| {
        if ret {
            if let Some(mapped) = app["destination"]
                .get_value()
                .downcast_ref::<&Vec<(String, String)>>()
            {
                mapped.iter().for_each(|(v, arg)| {
                    println!("Will copy location({}) to destination({})", v, arg);
                })
            }
        }
        Ok(())
    })
}
```

With input command argument `app.exe +S. --depth=2 --source ./src .. -/r`,
you will got output
```txt
Adding location(.) with depth(0)
Adding location(./src) with depth(2)
Save copy location(.) to destination(..)
Save copy location(./src) to destination(..)
Will copy location(.) to destination(..)
Will copy location(./src) to destination(..)
```

## Example2

With `getopt!` and `Parser`, you can match and process every command easily.

```rust
use aopt::err::Result;
use aopt::prelude::*;

fn main() -> Result<()> {
    let mut list = Parser::<SimpleSet, DefaultService, ForwardPolicy>::default();
    let mut update = Parser::<SimpleSet, DefaultService, ForwardPolicy>::default();
    let mut install = Parser::<SimpleSet, DefaultService, ForwardPolicy>::default();

    list.add_opt("list=c")?.commit()?;
    list.add_opt("ls=c")?.commit()?;
    list.add_opt("-debug=b")?.commit()?;
    list.add_opt("-force=b")?.add_alias("-f")?.commit()?;
    list.add_opt("-local-only=b")?.add_alias("-l")?.commit()?;
    list.add_opt("-source=s")?.add_alias("-s")?.commit()?;
    list.add_opt_cb(
        "main=m",
        simple_main_cb!(|_, set: &SimpleSet, _, value| {
            println!(
                "invoke list command: debug={:?}, force={:?}, local-only={:?}, source={:?}",
                set["debug"].get_value(),
                set["force"].get_value(),
                set["local-only"].get_value(),
                set["source"].get_value()
            );
            Ok(Some(value))
        }),
    )?
    .commit()?;

    update.add_opt("update=c")?.commit()?;
    update.add_opt("up=c")?.commit()?;
    update.add_opt("-debug=b")?.commit()?;
    update.add_opt("-force=b")?.add_alias("-f")?.commit()?;
    update.add_opt("-source=s")?.add_alias("-s")?.commit()?;
    update
        .add_opt_cb(
            "main=m",
            simple_main_cb!(|_, set: &SimpleSet, _, value| {
                println!(
                    "invoke update command: debug={:?}, force={:?}, source={:?}",
                    set["debug"].get_value(),
                    set["force"].get_value(),
                    set["source"].get_value()
                );
                Ok(Some(value))
            }),
        )?
        .commit()?;

    install.add_opt("install=c")?.commit()?;
    install.add_opt("in=c")?.commit()?;
    install.add_opt("-debug=b")?.commit()?;
    install.add_opt("-override=b/")?.add_alias("-o")?.commit()?;
    install.add_opt("-source=s")?.add_alias("-s")?.commit()?;
    install
        .add_opt_cb(
            "name=p!@2",
            simple_pos_cb!(|_, set: &SimpleSet, arg, _, value| {
                if arg == "software" {
                    println!(
                        "invoke install command: debug={:?}, override={:?}, source={:?}",
                        set["debug"].get_value(),
                        set["override"].get_value(),
                        set["source"].get_value()
                    );
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }),
        )?
        .commit()?;

    if getopt!(&mut std::env::args().skip(1), list, update, install)?.is_none() {
        println!("command not matched");
    }
    Ok(())
}
```

* `app.exe ls -source lib.rs -debug` output `invoke list command: debug=Bool(true), force=Null, local-only=Null, source=Str("lib.rs")`.

* `app.exe update -force -source=crates.io` output `invoke update command: debug=Null, force=Bool(true), source=Str("crates.io")`.

* `app.exe in software -/o -s crates.io` output `invoke install command: debug=Null, override=Bool(false), source=Str("crates.io")`.

* `app.exe in aopt` output `command not matched`.

### More

- simple-find-file

A simple file search tools, try it using [`cargo install --path simple-find-file`](https://github.com/araraloren/aopt/tree/main/simple-find-file).

- snowball-follow

Get the follow count of stock in `xueqiu.com`, try it using [`cargo install --path snowball-follow`](https://github.com/araraloren/aopt/tree/main/snowball-follow)

- index constituent

Search and list the constituent of index, try it using [`cargo install --path index-constituent`](https://github.com/araraloren/aopt/tree/main/index-constituent)

## LICENSE

MPL-2.0