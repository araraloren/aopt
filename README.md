# aopt

A flexible and typed getopt like command line tools for rust.

## Example

```rust
use aopt::err::Result;
use aopt::prelude::*;

fn main() -> Result<()> {
    let mut parser = SimpleParser::<UidGenerator>::default();
    let mut set = SimpleSet::default()
        .with_default_creator()
        .with_default_prefix();

    set.add_opt("-a=b!")?.commit()?;
    set.add_opt("--bopt=i")?.commit()?;
    set.add_opt("-c=a")?.add_alias("--copt")?.commit()?;
    parser.add_callback(
        set.add_opt("d=p@-1")?.commit()?,
        simple_pos_cb!(|_, _, arg, _, value| {
            assert_eq!(arg, "foo");
            Ok(Some(value))
        }),
    );

    let ret = getopt!(
        &mut ["-a", "-c", "foo", "--bopt=42", "foo", "--copt=bar"]
            .iter()
            .map(|&v| String::from(v)),
        set,
        parser
    )?;

    assert!(ret.is_some());
    assert_eq!(
        ret.as_ref().unwrap().set().get_value("-a")?,
        Some(&OptValue::from(true))
    );
    assert_eq!(
        ret.as_ref().unwrap().set().get_value("--bopt")?,
        Some(&OptValue::from(42i64))
    );
    assert_eq!(
        ret.as_ref().unwrap().set().get_value("--copt")?,
        Some(&OptValue::from(vec!["foo".to_owned(), "bar".to_owned()]))
    );
    Ok(())
}
```

### More

- simple-find-file

A simple file search tools, try it using [`cargo install --path simple-find-file`](https://github.com/araraloren/aopt/tree/main/simple-find-file).

- snowball-follow

Get the follow count of stock in `xueqiu.com`, try it using [`cargo install --path snowball-follow`](https://github.com/araraloren/aopt/tree/main/snowball-follow)

- index constituent

Search and list the constituent of index, try it using [`cargo install --path index-constituent`](https://github.com/araraloren/aopt/tree/main/index-constituent)

## LICENSE

MPL-2.0