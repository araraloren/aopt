# aopt

Generate help message for command line program.

## Example

```rust
fn display_help<S: Set>(set: &S) -> Result<(), aopt_help::Error> {
    let foot = format!(
        "Create by {} v{}",
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_VERSION")
    );
    let mut app_help = aopt_help::AppHelp::new(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_DESCRIPTION"),
        &foot,
        aopt_help::prelude::Style::default(),
        std::io::stdout(),
    );
    let global = app_help.global_mut();

    global.add_block(Block::new("option", "[OPTION]", "", "OPTION:", ""))?;
    global.add_block(Block::new("args", "[ARGS]", "", "ARGS:", ""))?;
    for opt in set.iter() {
        if opt.mat_style(Style::Pos) {
            global.add_store(
                "args",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::from(opt.r#type().to_string()),
                    opt.optional(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Argument)
            || opt.mat_style(Style::Boolean)
            || opt.mat_style(Style::Combined)
        {
            global.add_store(
                "option",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::from(opt.r#type().to_string()),
                    opt.optional(),
                    false,
                ),
            )?;
        }
    }

    app_help.display(true)?;

    Ok(())
}
```

## More 

- simple-find-file

A simple file search tools, try it using [`cargo install --path simple-find-file`](https://github.com/araraloren/aopt/tree/main/simple-find-file).

- snowball-follow

Get the follow count of stock in `xueqiu.com`, try it using [`cargo install --path snowball-follow`](https://github.com/araraloren/aopt/tree/main/snowball-follow)

## LICENSE

MPL-2.0