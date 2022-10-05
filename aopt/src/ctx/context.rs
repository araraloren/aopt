use crate::arg::Args;
use crate::opt::OptStyle;
use crate::Str;
use crate::Uid;

/// Invoke context using for [`InvokeService`](crate::ser::InvokeService).
#[derive(Debug, Clone, Default)]
pub struct Context {
    uid: Uid,

    name: Str,

    prefix: Option<Str>,

    style: OptStyle,

    deactivate: bool,

    argument: Option<Str>,

    args: Args,
}

impl Context {
    /// The uid of matching context.
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    /// The name of matching context.
    pub fn with_name(mut self, name: Str) -> Self {
        self.name = name;
        self
    }

    /// The prefix of matching context.
    pub fn with_prefix(mut self, prefix: Option<Str>) -> Self {
        self.prefix = prefix;
        self
    }

    /// The style of matching context.
    pub fn with_style(mut self, style: OptStyle) -> Self {
        self.style = style;
        self
    }

    /// The deactivate value of matching context.
    pub fn with_deactivate(mut self, deactivate: bool) -> Self {
        self.deactivate = deactivate;
        self
    }

    /// The argument of matching context.
    pub fn with_argument(mut self, argument: Option<Str>) -> Self {
        self.argument = argument;
        self
    }

    /// The arguments of matching context.
    pub fn with_args(mut self, args: Args) -> Self {
        self.args = args;
        self
    }

    pub fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    pub fn set_name(&mut self, name: Str) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_prefix(&mut self, prefix: Option<Str>) -> &mut Self {
        self.prefix = prefix;
        self
    }

    pub fn set_style(&mut self, style: OptStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_deactivate(&mut self, deactivate: bool) -> &mut Self {
        self.deactivate = deactivate;
        self
    }

    pub fn set_argument(&mut self, argument: Option<Str>) -> &mut Self {
        self.argument = argument;
        self
    }

    pub fn set_args(&mut self, args: Args) -> &mut Self {
        self.args = args;
        self
    }

    pub fn get_name(&self) -> Str {
        self.name.clone()
    }

    pub fn get_args(&self) -> &Args {
        &self.args
    }

    pub fn get_uid(&self) -> Uid {
        self.uid
    }

    pub fn get_index(&self) -> usize {
        self.args.get_index()
    }

    pub fn get_argument(&self) -> Option<Str> {
        self.argument.clone()
    }

    pub fn get_deactivate(&self) -> bool {
        self.deactivate
    }
}
