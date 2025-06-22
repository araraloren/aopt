pub mod script;
pub mod shell;
pub mod value;

pub(crate) use aopt_core as acore;

pub(crate) const SHELL_BASH: &str = "bash";
pub(crate) const SHELL_FISH: &str = "fish";
pub(crate) const SHELL_ZSH: &str = "zsh";
pub(crate) const SHELL_PSH: &str = "powershell";
pub(crate) const SHELL_PSH7: &str = "powershell7";

pub use acore::error;
pub use acore::failure;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

pub(crate) use acore::Error;

pub struct Context<'a> {
    pub args: &'a [OsString],

    /// Current argument passed by shell
    pub curr: Cow<'a, OsStr>,

    /// Previous argument passed by shell
    pub prev: Cow<'a, OsStr>,

    /// Index of current word
    pub cword: usize,
}

impl<'a> Context<'a> {
    pub fn new(args: &'a [OsString], curr: &'a OsString, prev: &'a OsString, cword: usize) -> Self {
        Self {
            args,
            curr: std::borrow::Cow::Borrowed(curr),
            cword,
            prev: std::borrow::Cow::Borrowed(prev),
        }
    }
}
