#![allow(unused)]
use std::ffi::OsString;
use std::path::PathBuf;

use cote::prelude::*;
use cote::shell::value::Values;

#[derive(Debug, Cote)]
#[cote(shellcomp)]
struct Cli {
    /// Enable IPV6 protocol
    ipv6: bool,

    /// Set http method
    #[arg(scvalues = ["PUT", "GET", "POST"])]
    method: Option<String>,

    /// Set server
    #[pos(scvalues = read_server_from_config())]
    sever: String,

    /// Set files
    #[pos(index = 2.., scvalues = list_files())]
    files: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if Cli::try_complete().is_ok() {
        return Ok(());
    }

    // do normal command line things
    let _ = Cli::parse_env()?;

    Ok(())
}

pub fn read_server_from_config<O>() -> impl Values<O> {
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
