#![allow(unused)]
use std::ffi::OsString;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Seek;
use std::path::PathBuf;

use cote::prelude::*;
use cote::shell::shell::Complete;
use cote::shell::value::Values;
use cote::shell::CompletionManager;
use cote::shell::Context;

#[derive(Debug, Cote)]
#[cote(shellcomp)]
struct Cli {
    /// Enable IPV6 protocol
    ipv6: bool,

    /// Set http method
    #[arg(scvalues = ["PUT", "GET", "POST"])]
    method: Option<String>,

    /// Set url
    #[pos(scvalues = read_url_from_config())]
    url: String,

    /// Set files
    #[pos(index = 2.., scvalues = list_files())]
    files: Vec<PathBuf>,
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

    assert!(complete_shells(
        vec!["example", "--method="],
        1,
        vec![
            (
                "fish",
                vec!["--method=PUT", "--method=GET", "--method=POST"]
            ),
            ("zsh", vec!["--method=PUT", "--method=GET", "--method=POST"]),
            ("bash", vec!["PUT", "GET", "POST"]),
            (
                "powershell",
                vec!["--method=PUT", "--method=GET", "--method=POST"]
            ),
            (
                "powershell7",
                vec!["--method=PUT", "--method=GET", "--method=POST"]
            )
        ]
    )
    .is_ok());

    assert!(complete_shells(
        vec!["example", "--ipv6", "--method", "PUT", ""],
        4,
        vec![
            ("fish", vec!["github.com", "gitlab.com"]),
            ("zsh", vec!["github.com", "gitlab.com"]),
            ("bash", vec!["github.com", "gitlab.com"]),
            ("powershell", vec!["github.com", "gitlab.com"]),
            ("powershell7", vec!["github.com", "gitlab.com"])
        ]
    )
    .is_ok());

    assert!(complete_shells(
        vec!["example"],
        1,
        vec![
            ("fish", vec!["github.com", "gitlab.com"]),
            ("zsh", vec!["github.com", "gitlab.com"]),
            ("bash", vec!["github.com", "gitlab.com"]),
            ("powershell", vec!["github.com", "gitlab.com"]),
            ("powershell7", vec!["github.com", "gitlab.com"])
        ]
    )
    .is_ok());

    assert!(complete_shells(
        vec!["example", "--ipv6", "--method", "PUT", "gitlab.com", ""],
        5,
        vec![
            ("fish", vec!["a.json", "b.toml", "c.yaml"]),
            ("zsh", vec!["a.json", "b.toml", "c.yaml"]),
            ("bash", vec!["a.json", "b.toml", "c.yaml"]),
            ("powershell", vec!["a.json", "b.toml", "c.yaml"]),
            ("powershell7", vec!["a.json", "b.toml", "c.yaml"])
        ]
    )
    .is_ok());

    Ok(())
}

pub fn read_url_from_config<O>() -> impl Values<O> {
    cote::shell::value::repeat_values(|_| {
        Ok(["github.com", "gitlab.com"]
            .iter()
            .map(OsString::from)
            .collect())
    })
}

pub fn list_files<O>() -> impl Values<O> {
    cote::shell::value::repeat_values(|_| {
        Ok(["a.json", "b.toml", "c.yaml"]
            .iter()
            .map(OsString::from)
            .collect())
    })
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
