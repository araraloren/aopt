
# Cote

A simple option manager manage the [`AOpt`](aopt::opt::AOpt), support auto generate help message.

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
    cote.add_opt("--from=i")?
        .add_alias("-f")
        .set_help("The sub string start index");
    cote.add_opt("--to=i")?
        .add_alias("-t")
        .set_help("The sub string end index");
    cote.run(
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