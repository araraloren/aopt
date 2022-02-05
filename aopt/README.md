# aopt

A flexible and typed getopt like command line tools for rust.

## Example

```rust
use aopt::err::Result;
use aopt::prelude::*;

fn main() -> Result<()> {
    let mut parser = Parser::<SimpleSet, DefaultService, ForwardPolicy>::default();

    parser.add_opt("-a=b!")?.commit()?;
    parser.add_opt("--bopt=i")?.commit()?;
    parser.add_opt("-c=a")?.add_alias("--copt")?.commit()?;
    parser
        .add_opt_cb(
            "d=p@-1",
            simple_pos_cb!(|_, _, arg, _, value| {
                assert_eq!(arg, "foo");
                Ok(Some(value))
            }),
        )?
        .commit()?;

    let ret = getopt!(
        ["-a", "-c", "foo", "--bopt=42", "foo", "--copt=bar"].into_iter(),
        parser
    )?;

    assert!(ret.is_some());
    let set = ret.as_ref().unwrap().get_set();
    assert_eq!(set.get_value("-a")?, Some(&OptValue::from(true)));
    assert_eq!(set.get_value("--bopt")?, Some(&OptValue::from(42i64)));
    assert_eq!(
        set.get_value("--copt")?,
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