# aopt-shell

Shell completion support for aopt framework.

## Example

```rust
use std::ffi::OsString;
use std::path::PathBuf;

use cote::prelude::*;
use cote::shell::get_complete_cli;
use cote::shell::shell::Complete;
use cote::shell::value::once_values;
use cote::shell::CompletionManager;

#[derive(Debug, Cote)]
pub struct Cli {
    /// Print debug message
    debug: bool,

    /// Set the count value of cli
    count: Option<i64>,

    /// Set the files of cli
    #[arg(alias = "-f")]
    files: Vec<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    match get_complete_cli() {
        Ok(cli) => {
            if cli.write_stdout("fput", "fput").is_err() {
                cli.complete(|shell| {
                    let mut ctx = cli.get_context()?;
                    let mut completion = CompletionManager::new(Cli::into_parser()?);

                    shell.set_buff(std::io::stdout());
                    completion.set_values(completion.optset().find_uid("--count")?, ["42", "56"]);
                    completion.set_values(
                        completion.optset().find_uid("--files")?,
                        once_values(|_| {
                            Ok(["files/a.txt", "files/b.txt", "files/c.txt"]
                                .map(OsString::from)
                                .into_iter()
                                .collect())
                        }),
                    );
                    completion.complete(shell, &mut ctx)?;
                    Ok(())
                })?;
            }
        }
        Err(_) => {
            let cli = Cli::parse_env()?;

            println!("doing normal cli things..: {cli:?}");
        }
    }

    Ok(())
}
```

## Deploy shell completions

PROGRAM is the name of binary

### bash

```bash
echo 'source <(PROGRAM --_shell bash)' >> ~/.bashrc
```

### fish

```fish
echo 'PROGRAM --_shell fish | source' >> ~/.config/fish/config.fish
```

### Zsh

```zsh
echo 'source <(PROGRAM --_shell zsh)' >> ~/.zshrc
```

### Powershell

```powershell
Add-Content $PROFILE "`nPROGRAM --_shell powershell | Out-String | Invoke-Expression"
```

## More 

- simple-find-file

A simple file search tools, try it using [`cargo install --path simple-find-file`](https://github.com/araraloren/aopt/tree/main/simple-find-file).

- snowball-follow

Get the follow count of stock in `xueqiu.com`, try it using [`cargo install --path snowball-follow`](https://github.com/araraloren/aopt/tree/main/snowball-follow)

## LICENSE

MPL-2.0