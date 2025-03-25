# aopt

A flexible and typed getopt like command line framwork for rust.

## Features

- Option support

    - Prefixed option support, such as `-f`, `--flag`, `-flag` or `--/flag`.

    - Option value support, such as `-f 42`, `--flag 3.14` or `--flag=foo`.

    - Multiple style option support, such as `-f 42`, `-f=42` or `-f42`.

    - Combing style support, such as `-abc` is same as `-a` `-b` `-c`.

    - Positional arguments support, see [`Index`](crate::opt::Index).

    - Special option `-` and `--` support, see [`Stop`](crate::value::Stop) and [`Stdin`](std::io::Stdin).

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

    - [`SeqPolicy`](crate::parser::SeqPolicy) process all type arguments one by one.

- Derive support

    - Checkout [`cote`](https://docs.rs/cote/latest/cote/index.html) crate for derive support and help message generate.

## Setup 

`cargo add aopt`

### `sync` feature

If you want the utils of current crate implement `Send` and `Sync`, you can enable `sync` feature.

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

## Examples

### Using [`AFwdParser`](crate::prelude::AFwdParser) parsing process the command line.

```rust ,no_run
use aopt::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = AFwdParser::default();

    parser.validator_mut().add_prefix("+");
    parser.add_opt("--depth=i")?.set_value_t(0i64); // int option
    parser.add_opt("-/r=b")?; // boolean flag
    parser
        .add_opt("--source=s!")? // ! means the option is force required
        .add_alias("+S")
        .on(|set, ctx| {
            let val = ctx.value::<String>()?;
            let depth: &i64 = set["--depth"].val()?;
            println!("Adding location({}) with depth({})", val, depth);
            Ok(Some((val, *depth)))
        })?;
    parser.add_opt("destination=p!@-0")?.on(|_, ctx| {
        let val = ctx.value::<String>()?;
        println!("Save destination location({})", val);
        Ok(Some(val))
    })?;
    parser.add_opt("main=m")?.on(|set, ctx| {
        let val = ctx.value::<String>()?;
        let src = set["--source"].vals::<(String, i64)>()?;
        let dest: &String = set["destination"].val()?;

        for (item, depth) in src {
            println!(
                "Application {} will copy location({item}, depth={depth}) to destination({})",
                val, dest
            );
        }
        Ok(Some(val))
    })?;
    parser.parse_env()?.unwrap();

    Ok(())
}
```

* `app.exe --depth=98 +S github --depth=42 +S gitlab gitcode` output

```txt
Adding location(github) with depth(98)
Adding location(gitlab) with depth(42)
Save destination location(gitcode)
Application target\debug\example.exe will copy location(github, depth=98) to destination(gitcode)
Application target\debug\example.exe will copy location(gitlab, depth=42) to destination(gitcode)
```

### Using [`getopt!`](crate::getopt) parsing multiple sub command.

```rust ,no_run
use aopt::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut list = AFwdParser::default();
    let mut update = AFwdParser::default();
    let mut install = AFwdParser::default();

    list.add_opt("list=c")?;
    list.add_opt("ls=c")?;
    list.add_opt("-debug=b")?;
    list.add_opt("-force=b")?.add_alias("-f");
    list.add_opt("-local-only=b")?.add_alias("-l");
    list.add_opt("-source".infer::<String>())?
        .add_alias("-s")
        .set_value(String::from("lib.rs"));
    list.add_opt("main=m")?.fallback(|set, _| {
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
    update.add_opt("main=m")?.on(|set, _| {
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
    install.add_opt("name=p!@2")?.on(|set, ctx| {
        let val = ctx.value::<String>()?;
        if val == "software" {
            println!(
                "invoke install command: debug={:?}, override={:?}, source={:?}",
                set["-debug"].val::<bool>()?,
                set["-/override"].val::<bool>()?,
                set["-source"].val::<String>()?,
            );
            Ok(Some(val))
        } else {
            Err(aopt::raise_error!("command not matched"))
        }
    })?;

    getopt!(Args::from_env(), &mut list, &mut update, &mut install)?;
    Ok(())
}
```

* `app.exe ls -debug` output

    invoke list command: debug=true, force=false, local-only=false, source="lib.rs"

* `app.exe update -force -source=crates.io` output

    invoke update command: debug=false, force=true, source="crates.io"

* `app.exe in software -/o -s crates.io` output

    invoke install command: debug=false, override=true, source="crates.io"

* `app.exe in aopt` output

    Error: command not matched

### Can support different policy 

```rust
use aopt::prelude::*;
use aopt::raise_error;
use aopt::value::raw2str;
use aopt::Error;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug, Default, PartialEq, Eq)]
    pub enum Magic {
        Upper,
        #[default]
        Lower,
        Camel,
    }

    impl Magic {
        pub fn piu(&self, val: &str) -> String {
            match self {
                Magic::Upper => val.to_uppercase(),
                Magic::Lower => val.to_lowercase(),
                Magic::Camel => {
                    let val = val.to_lowercase();
                    let mut chars = val.chars();

                    format!(
                        "{}{}",
                        chars.next().unwrap_or_default().to_uppercase(),
                        chars.collect::<String>()
                    )
                }
            }
        }
    }

    // Implement RawValParser for type Magic
    impl RawValParser for Magic {
        type Error = Error;

        fn parse(raw: Option<&OsStr>, _: &Ctx) -> Result<Self, Self::Error> {
            let mode = raw2str(raw)?.to_lowercase();

            match mode.as_str() {
                "upper" => Ok(Self::Upper),
                "lower" => Ok(Self::Lower),
                "camel" => Ok(Self::Camel),
                _ => Err(raise_error!("What do you want? wahaha")),
            }
        }
    }

    let mut set = AHCSet::default();

    // using MutOpt for type only implement RawValParser
    set.add_opt("-m;--magic".infer::<MutOpt<Magic>>())?;
    set.add_opt("file@*".infer::<Pos<String>>())?
        .on(|set, ctx| {
            let val = ctx.value::<String>()?;
            let magic = set.take_val::<Magic>("-m").unwrap_or_default();

            Ok(Some(magic.piu(&val)))
        })?;

    let args = [
        "app", "-mcamel", "html", "--magic", "lower", "png", "TXT", "-m=upper", "mp4",
    ];

    // using seq policy
    let mut seq = ASeqPolicy::default();

    assert!(set.parse_policy(Args::from(args), &mut seq)?.ok().is_ok());
    assert_eq!(
        set.take_vals::<String>("file")?,
        &["Html", "png", "txt", "MP4"]
    );

    // using fwd policy
    let mut fwd = AFwdPolicy::default();

    assert!(set.parse_policy(Args::from(args), &mut fwd)?.ok().is_ok());
    assert_eq!(
        set.take_vals::<String>("file")?,
        &["HTML", "png", "Txt", "mp4"]
    );

    // using delay policy
    let mut delay = ADelayPolicy::default();

    assert!(set.parse_policy(Args::from(args), &mut delay)?.ok().is_ok());
    assert_eq!(
        set.take_vals::<String>("file")?,
        &["html", "png", "txt", "mp4"]
    );

    Ok(())
}
```

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