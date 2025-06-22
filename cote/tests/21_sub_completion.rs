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

pub const OS: [&str; 4] = ["HarmonyOS", "Windows", "Android", "MacOS"];

#[derive(Debug, Cote)]
#[cote(shellcomp)]
struct System {
    /// Enable user flag
    preserve_user: bool,

    /// Migrate from on OS to another OS
    #[sub(scvalues)]
    migrate: Option<Migrate>,

    /// Install OS from server
    #[sub(scvalues)]
    install: Option<Install>,

    /// Update OS
    #[sub()]
    update: Option<Update>,
}

#[derive(Debug, Cote)]
pub struct Migrate {
    /// Set the name of OS
    #[pos(scvalues = OS)]
    from: Option<String>,

    /// Set the name of OS
    #[pos(scvalues = OS)]
    to: Option<String>,
}

#[derive(Debug, Cote)]
pub struct Install {
    /// Set the OS server
    #[arg(scvalues = read_server_from_cfg())]
    url: Option<String>,

    /// Set the name of OS
    #[pos(scvalues = OS)]
    name: Option<String>,
}

#[derive(Debug, Cote)]
pub struct Update {
    /// Enable yes flag
    yes: bool,

    /// Enable only flag
    only: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert!(complete_shells(
        vec!["example", "migrate"],
        2,
        vec![
            ("fish", OS.to_vec()),
            ("zsh", OS.to_vec()),
            ("bash", OS.to_vec()),
            ("powershell", OS.to_vec()),
            ("powershell7", OS.to_vec())
        ]
    )
    .is_ok());

    assert!(complete_shells(
        vec!["example", "migrate", "Harm"],
        2,
        vec![
            ("fish", vec!["HarmonyOS"]),
            ("zsh", vec!["HarmonyOS"]),
            ("bash", vec!["HarmonyOS"]),
            ("powershell", vec!["HarmonyOS"]),
            ("powershell7", vec!["HarmonyOS"]),
        ]
    )
    .is_ok());

    assert!(complete_shells(
        vec!["example", "migrate", "HarmonyOS"],
        3,
        vec![
            ("fish", OS.to_vec()),
            ("zsh", OS.to_vec()),
            ("bash", OS.to_vec()),
            ("powershell", OS.to_vec()),
            ("powershell7", OS.to_vec())
        ]
    )
    .is_ok());

    Ok(())
}

pub fn read_server_from_cfg<O>() -> impl Values<O> {
    cote::shell::value::repeat_values(|_| {
        Ok(["a.com/iso", "b.org/iso", "c.io/iso"]
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
    let mut manager = CompletionManager::new(System::into_parser()?);

    shell.set_buff(writer);
    System::inject_completion_values(&mut manager)?;
    assert!(manager.complete(shell, &mut ctx).is_ok());
    let mut cursor = shell.take_buff().unwrap();

    cursor.seek(std::io::SeekFrom::Start(0))?;

    let output: Vec<_> = cursor.lines().collect::<Result<Vec<_>, std::io::Error>>()?;

    assert_eq!(output, except, "completion output check failed!");
    Ok(())
}
