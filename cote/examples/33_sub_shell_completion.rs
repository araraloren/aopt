#![allow(unused)]
use std::ffi::OsString;

use aopt::shell::get_complete_cli;
use aopt::shell::value::once_values;
use cote::prelude::*;
use cote::shell::shell::Complete;
use cote::shell::value::Values;
use cote::shell::CompletionManager;

pub const OS: [&str; 4] = ["HarmonyOS", "Windows", "Android", "MacOS"];

#[derive(Debug, Cote)]
#[cote(shellcomp)]
struct OperateSystem {
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
    /// Set the OS server
    #[arg(scvalues = read_server_from_cfg())]
    server: Option<String>,

    /// Set the name of OS
    #[pos()]
    from: Option<String>,

    /// Set the name of OS
    #[pos()]
    to: Option<String>,
}

#[derive(Debug, Cote)]
pub struct Install {
    /// Set the OS server
    #[arg(scvalues = read_server_from_cfg())]
    server: Option<String>,

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
    // pass the executable binary name
    if try_complete("sub_shell_completion").is_ok() {
        return Ok(());
    }

    // do normal command line things
    let _ = OperateSystem::parse_env()?;

    Ok(())
}

pub fn try_complete(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cli = get_complete_cli()?;

    if cli.write_stdout(name, name).is_ok() {
        return Ok(());
    }
    cli.complete(|shell| {
        let mut ctx = cli.get_context()?;
        let mut parser = OperateSystem::into_parser()?;
        let mut policy = OperateSystem::into_policy().with_prepolicy(true);

        // process arguments
        let _ = parser.parse_policy(Args::from(&cli.args), &mut policy);

        // get Migrate of from parser
        let migr = parser.take_val::<Migrate>("migrate").ok();
        let from = migr.and_then(|v| v.from);

        let mut manager = CompletionManager::new(parser);
        let migr = manager.find_manager_mut("migrate")?;

        // set values of from and to
        migr.set_values(migr.parser().find_uid("from")?, OS);
        migr.set_values(migr.parser().find_uid("to")?, list_os_except(from));

        // inject other values
        OperateSystem::inject_completion_values(&mut manager)?;

        // set buff of shell
        shell.set_buff(std::io::stdout());

        // do complete
        manager.complete(shell, &mut ctx)?;
        Ok(())
    })?;
    Ok(())
}

pub fn read_server_from_cfg<O>() -> impl Values<O> {
    once_values(|_| {
        Ok(["a.com", "b.org", "c.io"]
            .iter()
            .map(OsString::from)
            .collect())
    })
}

pub fn list_os_except<O>(from: Option<String>) -> impl Values<O> {
    once_values(move |_| {
        let mut vals = OS.to_vec();

        if let Some(idx) = from.as_ref().and_then(|r| vals.iter().position(|v| v == r)) {
            vals.swap_remove(idx);
        }
        Ok(vals.iter().map(OsString::from).collect())
    })
}
