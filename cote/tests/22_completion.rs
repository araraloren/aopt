#![allow(unused)]

use std::ffi::OsString;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Seek;

use cote::prelude::*;
use cote::shell::shell::Complete;
use cote::shell::CompletionManager;
use cote::shell::Context;

#[derive(Debug, Cote)]
#[cote(shellcomp)]
struct Cli {
    /// Enable IPV6 protocol
    ipv6: bool,

    /// Set http method
    method: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert!(complete_shells(
        vec!["example", "--i"],
        1,
        vec![
            ("fish", vec!["--ipv6\t\"Enable IPV6 protocol\""]),
            ("zsh", vec!["--ipv6:Enable IPV6 protocol"]),
            ("bash", vec!["--ipv6"]),
            ("powershell", vec!["--ipv6\tEnable IPV6 protocol"]),
            ("powershell7", vec!["--ipv6\tEnable IPV6 protocol"])
        ]
    )
    .is_ok());

    Ok(())
}

pub fn complete_shells(
    args: Vec<impl Into<OsString>>,
    cword: usize,
    excepts: Vec<(&str, Vec<impl Into<String>>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = args.into_iter().map(|v| v.into()).collect::<Vec<_>>();

    for (shell, except) in excepts {
        let except = except.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        complete_at(shell, &args, cword, &except)?;
    }
    Ok(())
}

pub fn complete_at(
    shell: &str,
    args: &[OsString],
    cword: usize,
    except: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let os_string = OsString::default();
    let writer = Cursor::new(vec![]);
    let mut ctx = Context::new(
        args,
        args.get(cword).unwrap_or(&os_string),
        &args[cword - 1],
        cword,
    );
    let mut m = cote::shell::shell::Manager::default();
    let shell = m.find_mut(shell)?;
    let mut manager = CompletionManager::new(Cli::into_parser()?);

    shell.set_buff(writer);
    Cli::inject_completion_values(&mut manager)?;
    assert!(manager.complete(shell, &mut ctx).is_ok());
    let mut cursor = shell.take_buff().unwrap();

    cursor.seek(std::io::SeekFrom::Start(0))?;

    let output: Vec<_> = cursor.lines().collect::<Result<Vec<_>, std::io::Error>>()?;

    assert_eq!(output, except, "completion output check failed!");
    Ok(())
}
