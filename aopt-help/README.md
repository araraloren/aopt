# aopt

Generate help message for aopt.

## Example

```rust
fn simple_help_generate(set: &dyn Set) -> AppHelp<Stdout, DefaultFormat> {
    let mut help = AppHelp::default();

    help.set_name("snowball".into());

    let global = help.store.get_global_mut();

    for opt in set.opt_iter() {
        if opt.match_style(aopt::opt::Style::Pos) {
            global.add_pos(PosStore::new(
                opt.get_name(),
                opt.get_hint(),
                opt.get_help(),
                opt.get_index().unwrap().to_string().into(),
                opt.get_optional(),
            ));
        } else if !opt.match_style(aopt::opt::Style::Main) {
            global.add_opt(OptStore::new(
                opt.get_name(),
                opt.get_hint(),
                opt.get_help(),
                opt.get_type_name(),
                opt.get_optional(),
            ));
        }
    }

    global.set_header(gstr("Get the follow people number in https://xueqiu.com/"));
    global.set_footer(gstr(&format!(
        "Create by araraloren {}",
        env!("CARGO_PKG_VERSION")
    )));

    help
}
```

## LICENSE

MPL-2.0