pub mod script;
pub mod shell;
pub mod value;

pub(crate) use aopt_core as acore;

pub use acore::error;
pub use acore::failure;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

use acore::Error;
use acore::HashMap;
use acore::opt::Opt;

use crate::value::Values;

pub struct Context<'a, O>
where
    O: Opt,
{
    pub args: &'a [OsString],

    /// Current argument passed by shell
    pub arg: Cow<'a, OsStr>,

    /// Value of current argument passed by shell
    pub val: Option<Cow<'a, OsStr>>,

    /// Previous argument passed by shell
    pub prev: Cow<'a, OsStr>,

    /// Values of options
    pub values: HashMap<String, Box<dyn Values<O, Err = Error>>>,
}

impl<'a, O> Context<'a, O>
where
    O: Opt,
{
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
            values: HashMap::default(),
        }
    }

    pub fn with_values<V>(mut self, name: &str, v: V) -> Self
    where
        V: Values<O> + 'static,
    {
        self.set_values(name, v);
        self
    }

    pub fn set_values<V>(&mut self, name: &str, v: V) -> &mut Self
    where
        V: Values<O> + 'static,
    {
        self.values
            .insert(name.to_string(), Box::new(crate::value::wrap(v)));
        self
    }
}
