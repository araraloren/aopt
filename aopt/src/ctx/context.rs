use crate::args::Args;
use crate::opt::Style;
use crate::Arc;
use crate::RawVal;
use crate::Str;
use crate::Uid;

/// The invoke context of option handler.
/// It saved the option information and matched arguments.
#[derive(Debug, Clone, Default)]
pub struct Ctx {
    uid: Uid,

    name: Option<Str>,

    style: Style,

    arg: Option<Arc<RawVal>>,

    index: usize,

    total: usize,

    args: Arc<Args>,
}

impl Ctx {
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    pub fn with_idx(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_total(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn with_args(mut self, args: Arc<Args>) -> Self {
        self.args = args;
        self
    }

    pub fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_arg(mut self, arg: Option<Arc<RawVal>>) -> Self {
        self.arg = arg;
        self
    }
}

impl Ctx {
    /// The uid of matched option.
    pub fn uid(&self) -> Uid {
        self.uid
    }

    /// The index of matched option.
    pub fn idx(&self) -> usize {
        self.index
    }

    /// The total number of arguments.
    pub fn total(&self) -> usize {
        self.total
    }

    /// The name of matched option.
    /// For option it is the option name, for NOA it is the argument,
    /// which set in [`guess`](crate::parser::Guess::guess).
    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    /// The style of matched option.
    pub fn style(&self) -> Style {
        self.style
    }

    /// The copy of [`Args`] when the option matched.
    pub fn args(&self) -> &Arc<Args> {
        &self.args
    }

    /// The argument which set in [`guess`](crate::parser::Guess::guess).
    pub fn arg(&self) -> Option<Arc<RawVal>> {
        self.arg.clone()
    }

    /// The first argument from [`Args`].
    pub fn orig_arg(&self) -> Option<&RawVal> {
        (self.idx() > 0)
            .then(|| self.args().get(self.idx().saturating_sub(1)))
            .flatten()
    }
}

impl Ctx {
    pub fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    /// The index of matching context.
    pub fn set_idx(&mut self, index: usize) -> &mut Self {
        self.index = index;
        self
    }

    /// The total of matching context.
    pub fn set_total(&mut self, total: usize) -> &mut Self {
        self.total = total;
        self
    }

    pub fn set_args(&mut self, args: Arc<Args>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn set_name(&mut self, name: Option<Str>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_arg(&mut self, argument: Option<Arc<RawVal>>) -> &mut Self {
        self.arg = argument;
        self
    }
}
