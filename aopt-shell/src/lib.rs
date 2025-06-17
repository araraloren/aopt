pub mod script;
pub mod shell;
pub mod value;

pub(crate) use aopt_core as acore;

pub(crate) const SHELL_BASH: &str = "bash";
pub(crate) const SHELL_FISH: &str = "fish";
pub(crate) const SHELL_ZSH: &str = "zsh";
pub(crate) const SHELL_PSH: &str = "powershell";

pub use acore::error;
pub use acore::failure;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

pub(crate) use acore::Error;

pub struct Context<'a> {
    pub args: &'a [OsString],

    /// Current argument passed by shell
    pub arg: Cow<'a, OsStr>,

    /// Value of current argument passed by shell
    pub val: Option<Cow<'a, OsStr>>,

    /// Previous argument passed by shell
    pub prev: Cow<'a, OsStr>,
}

impl<'a> Context<'a> {
    pub fn new(args: &'a [OsString], curr: &'a OsString, prev: &'a OsString) -> Self {
        use std::borrow::Cow;

        let mut incomplete_arg = Cow::Borrowed(curr.as_ref());
        let mut incomplete_val = None;

        if let Some((opt, val)) = aopt_core::str::split_once(curr, '=') {
            incomplete_arg = opt;
            incomplete_val = Some(val);
        }

        Self {
            args,
            arg: incomplete_arg,
            val: incomplete_val,
            prev: std::borrow::Cow::Borrowed(prev),
        }
    }
}
